//! Neutral provider-context projection policy.
//!
//! This module describes and hashes a public projection.  It never reads
//! provider state, handles keys, records audits, or authorizes transmission.

use alloc::{vec, vec::Vec};
use sha2::{Digest, Sha256};

use crate::{
    method_eq, method_head_eq,
    record::sha256_of_json,
    record::{Field, Value},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionFieldSpec {
    pub field: &'static str,
    pub classification: &'static str,
    pub action: &'static str,
    pub reason: &'static str,
}

pub const PROVIDER_MINIMAL_INCLUDED_FIELDS: &[ProjectionFieldSpec] = &[
    ProjectionFieldSpec {
        field: "schema",
        classification: "public",
        action: "include",
        reason: "context schema id is non-secret and required for decoding",
    },
    ProjectionFieldSpec {
        field: "purpose",
        classification: "public",
        action: "include",
        reason: "bounded task purpose, not raw user prompt",
    },
    ProjectionFieldSpec {
        field: "profile",
        classification: "public",
        action: "include",
        reason: "redaction profile id",
    },
    ProjectionFieldSpec {
        field: "scope",
        classification: "public",
        action: "include",
        reason: "current_boot scope marker",
    },
    ProjectionFieldSpec {
        field: "budget.target_tokens",
        classification: "public",
        action: "include",
        reason: "token budget target for the projection",
    },
    ProjectionFieldSpec {
        field: "budget.estimated_tokens",
        classification: "public",
        action: "include",
        reason: "bounded estimate only",
    },
    ProjectionFieldSpec {
        field: "authority_order[]",
        classification: "public",
        action: "include",
        reason: "authority labels from ADR 0004",
    },
    ProjectionFieldSpec {
        field: "included.*[]",
        classification: "public",
        action: "include",
        reason: "stable record ids only",
    },
    ProjectionFieldSpec {
        field: "current.os.*",
        classification: "public",
        action: "include",
        reason: "product identity and stage",
    },
    ProjectionFieldSpec {
        field: "current.status.*",
        classification: "public",
        action: "include",
        reason: "coarse subsystem states only",
    },
    ProjectionFieldSpec {
        field: "current.provider.selected",
        classification: "public",
        action: "include",
        reason: "provider family only",
    },
    ProjectionFieldSpec {
        field: "current.provider.route",
        classification: "public",
        action: "include",
        reason: "canonical route name without credentials",
    },
    ProjectionFieldSpec {
        field: "current.provider.api_key_state",
        classification: "public",
        action: "include",
        reason: "state marker only; never the key",
    },
    ProjectionFieldSpec {
        field: "current.provider.direct_phase",
        classification: "public",
        action: "include",
        reason: "coarse provider phase",
    },
    ProjectionFieldSpec {
        field: "current.provider.direct_endpoint",
        classification: "public",
        action: "include",
        reason: "canonical provider endpoint",
    },
    ProjectionFieldSpec {
        field: "current.provider.direct_model",
        classification: "public",
        action: "include",
        reason: "model id",
    },
    ProjectionFieldSpec {
        field: "current.provider.trust_state",
        classification: "public",
        action: "include",
        reason: "required for fail-closed provider policy",
    },
    ProjectionFieldSpec {
        field: "current.provider.pin_kind",
        classification: "public",
        action: "include",
        reason: "pin type only",
    },
    ProjectionFieldSpec {
        field: "current.provider.pin_id",
        classification: "public",
        action: "include",
        reason: "short non-secret pin identifier only",
    },
    ProjectionFieldSpec {
        field: "current.provider.pin_slot",
        classification: "public",
        action: "include",
        reason: "active or rotation slot marker only",
    },
    ProjectionFieldSpec {
        field: "current.provider.pin_rotation_policy",
        classification: "public",
        action: "include",
        reason: "explicit pin-only rotation posture",
    },
    ProjectionFieldSpec {
        field: "current.provider.pin_rotation_id",
        classification: "public",
        action: "include",
        reason: "short non-secret standby pin identifier only",
    },
    ProjectionFieldSpec {
        field: "current.provider.development_bypass",
        classification: "public",
        action: "include",
        reason: "must be visible because it blocks trusted export",
    },
    ProjectionFieldSpec {
        field: "current.provider.verifier_decision.*",
        classification: "public",
        action: "include",
        reason: "typed trust verifier outcome without certificate bytes or secrets",
    },
    ProjectionFieldSpec {
        field: "current.services[]",
        classification: "public",
        action: "include",
        reason: "stable service ids only",
    },
    ProjectionFieldSpec {
        field: "current.capabilities[]",
        classification: "public",
        action: "include",
        reason: "capability ids and denied mutation posture",
    },
    ProjectionFieldSpec {
        field: "current.problems[].id",
        classification: "public",
        action: "include",
        reason: "stable problem ids",
    },
    ProjectionFieldSpec {
        field: "current.problems[].severity",
        classification: "public",
        action: "include",
        reason: "coarse severity",
    },
    ProjectionFieldSpec {
        field: "current.problems[].summary",
        classification: "public",
        action: "include",
        reason: "stable scrubbed summaries only",
    },
    ProjectionFieldSpec {
        field: "records[].id",
        classification: "public",
        action: "include",
        reason: "stable locators",
    },
    ProjectionFieldSpec {
        field: "records[].kind",
        classification: "public",
        action: "include",
        reason: "record kind labels",
    },
    ProjectionFieldSpec {
        field: "records[].authority",
        classification: "public",
        action: "include",
        reason: "authority labels",
    },
    ProjectionFieldSpec {
        field: "records[].classification",
        classification: "public",
        action: "include",
        reason: "classification labels",
    },
    ProjectionFieldSpec {
        field: "records[].summary",
        classification: "public",
        action: "include",
        reason: "only summaries for records classified public",
    },
];

pub const PROVIDER_MINIMAL_OMITTED_FIELDS: &[ProjectionFieldSpec] = &[
    ProjectionFieldSpec {
        field: "source_schemas",
        classification: "local_only",
        action: "omit",
        reason: "local provenance list; the provider packet names only its projection schema",
    },
    ProjectionFieldSpec {
        field: "system.snapshot.raw",
        classification: "local_only",
        action: "omit",
        reason: "raw system.snapshot is never attached to provider context",
    },
    ProjectionFieldSpec {
        field: "details.*.detail",
        classification: "local_only",
        action: "omit",
        reason: "detail strings may contain IPs, PCI data, topology, request ids, or hashes",
    },
    ProjectionFieldSpec {
        field: "network.ip",
        classification: "local_only",
        action: "omit",
        reason: "local network address",
    },
    ProjectionFieldSpec {
        field: "network.gateway",
        classification: "local_only",
        action: "omit",
        reason: "local network topology",
    },
    ProjectionFieldSpec {
        field: "network.dns",
        classification: "local_only",
        action: "omit",
        reason: "local resolver topology",
    },
    ProjectionFieldSpec {
        field: "provider.direct_last_prompt",
        classification: "secret",
        action: "omit",
        reason: "raw prompt text is not context memory",
    },
    ProjectionFieldSpec {
        field: "provider.direct_last_error",
        classification: "local_only",
        action: "omit",
        reason: "error text may contain request ids or provider diagnostics",
    },
    ProjectionFieldSpec {
        field: "provider.direct_last_event",
        classification: "local_only",
        action: "omit",
        reason: "free-form event text remains local",
    },
    ProjectionFieldSpec {
        field: "provider.direct_pending_id",
        classification: "local_only",
        action: "omit",
        reason: "local request correlation id",
    },
    ProjectionFieldSpec {
        field: "provider.direct_last_request_id",
        classification: "local_only",
        action: "omit",
        reason: "local request correlation id",
    },
    ProjectionFieldSpec {
        field: "provider.tcp.*",
        classification: "local_only",
        action: "omit",
        reason: "TCP diagnostics include local transport details",
    },
    ProjectionFieldSpec {
        field: "wifi.ssid",
        classification: "secret",
        action: "omit",
        reason: "raw SSID is user-local configuration",
    },
    ProjectionFieldSpec {
        field: "wifi.passphrase",
        classification: "secret",
        action: "omit",
        reason: "raw Wi-Fi passphrase is never exported",
    },
    ProjectionFieldSpec {
        field: "system.boot_log.raw",
        classification: "local_only",
        action: "omit",
        reason: "raw boot log remains behind local-only methods",
    },
    ProjectionFieldSpec {
        field: "current.recovery_lifeline_status",
        classification: "local_only",
        action: "omit",
        reason:
            "recovery lifeline status fact is local current-boot evidence, not provider context",
    },
    ProjectionFieldSpec {
        field: "recovery.lifeline.status.current_boot",
        classification: "local_only",
        action: "omit",
        reason: "local recovery status locator is authority-bearing evidence, not provider context",
    },
    ProjectionFieldSpec {
        field: "boot_log.summary.current",
        classification: "local_only",
        action: "omit",
        reason: "boot-log summary is a local locator, not provider context",
    },
    ProjectionFieldSpec {
        field: "records[].source",
        classification: "local_only",
        action: "omit",
        reason: "source file paths and local method names remain trace-only locators",
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProviderProjectionField<'a> {
    pub field: &'a str,
    pub value: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProviderProjectionRecord<'a> {
    pub id: &'a str,
    pub kind: &'a str,
    pub entity: &'a str,
    pub predicate: &'a str,
    pub classification: &'a str,
    pub authority: &'a str,
    pub exportable: bool,
}

pub struct ProviderProjectionInput<'a> {
    pub status: &'a [ProviderProjectionField<'a>],
    pub provider: &'a [ProviderProjectionField<'a>],
    pub services: &'a [&'a str],
    pub capabilities: &'a [&'a str],
    pub problems: &'a [ProviderProjectionField<'a>],
    pub records: &'a [ProviderProjectionRecord<'a>],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProviderContextHashes {
    pub projected_packet_hash: [u8; 32],
    pub exported_field_list_hash: [u8; 32],
    pub omitted_field_list_hash: [u8; 32],
    pub redaction_policy_hash: [u8; 32],
    pub field_classification_hash: [u8; 32],
    pub token_budget_hash: [u8; 32],
}

pub fn provider_context_export_method(method: &str) -> bool {
    method_head_eq(method, "provider.context_export")
        || method_head_eq(method, "provider.export_context")
}
pub fn provider_context_export_profile(method: &str) -> &'static str {
    let arg = provider_context_export_arg(method);
    if method_eq(arg, "provider_minimal") || arg.is_empty() {
        "provider_minimal"
    } else {
        "unsupported"
    }
}
pub fn provider_trust_positive(trust_state: &str) -> bool {
    matches!(
        trust_state,
        "pinned_cert_verified" | "pinned_spki_verified" | "webpki_verified"
    )
}
pub fn provider_context_block_reason(trust_state: &str) -> &'static str {
    if provider_trust_positive(trust_state) {
        "provider_context_export_audit_binding_missing"
    } else {
        "provider_trust_not_positive"
    }
}
pub fn provider_context_export_arg(method: &str) -> &str {
    let method = method.trim();
    let names = [
        "provider.context_export",
        "provider.export_context",
        "provider.context_gate",
        "provider.context_export_status",
        "provider.context_gate_selftest",
        "provider.context_injection_gate",
        "provider.context_injection_gate_selftest",
        "provider.context_export_packet_selftest",
        "provider.context_export_authorized_selftest",
        "provider.context_export_authorized_selftest_smuggle",
    ];
    let mut i = 0;
    while i < names.len() {
        if method_head_eq(method, names[i]) {
            return method[names[i].len()..].trim();
        }
        i += 1;
    }
    ""
}

pub fn projection_field_list_hash(
    domain: &'static str,
    fields: &[ProjectionFieldSpec],
) -> [u8; 32] {
    let mut h = EvidenceHash::new(domain);
    for field in fields {
        h.field("field", field.field);
        h.field("classification", field.classification);
        h.field("action", field.action);
        h.field("reason", field.reason);
        h.separator();
    }
    h.finish()
}
pub fn provider_redaction_policy_hash() -> [u8; 32] {
    let mut h = EvidenceHash::new("raios.provider_minimal.redaction_policy.canonical.v0");
    h.field("profile", "provider_minimal");
    h.field("redaction_projection", "present");
    h.field("unclassified_field_policy", "omit");
    h.field("secret_field_policy", "omit");
    h.field("local_only_field_policy", "omit");
    h.field("provider_write", "not_attempted");
    h.finish()
}
pub fn provider_field_classification_hash() -> [u8; 32] {
    let mut h = EvidenceHash::new("raios.provider_minimal.field_classification.canonical.v0");
    for fields in [
        PROVIDER_MINIMAL_INCLUDED_FIELDS,
        PROVIDER_MINIMAL_OMITTED_FIELDS,
    ] {
        for f in fields {
            h.field("field", f.field);
            h.field("classification", f.classification);
            h.field("action", f.action);
            h.separator();
        }
    }
    h.finish()
}
pub fn provider_token_budget_hash() -> [u8; 32] {
    let mut h = EvidenceHash::new("raios.provider_minimal.token_budget.canonical.v0");
    h.field("profile", "provider_minimal");
    h.field("budget.target_tokens", "2000");
    h.field("budget.estimated_tokens", "900");
    h.field("budget.enforcement", "bounded_projection");
    h.finish()
}

pub fn provider_export_packet_value<'a>(records: &[ProviderProjectionRecord<'a>]) -> Value<'a> {
    let mut values = Vec::new();
    for record in records {
        if record.exportable {
            values.push(Value::InlineObject(vec![
                f("id", Value::Str(record.id)),
                f("kind", Value::Str(record.kind)),
                f("entity", Value::Str(record.entity)),
                f("predicate", Value::Str(record.predicate)),
                f("classification", Value::Str(record.classification)),
                f("authority", Value::Str(record.authority)),
                f("scope", Value::Str("durable")),
                f("exportable", Value::Bool(true)),
            ]));
        }
    }
    Value::Object(vec![
        f("profile", Value::Str("provider_minimal")),
        f("scope", Value::Str("durable")),
        f("packet_all_records_public", Value::Bool(true)),
        f("packet_record_count", Value::U64(values.len() as u64)),
        f("records", Value::Array(values)),
    ])
}
pub fn provider_export_packet_hash(records: &[ProviderProjectionRecord<'_>]) -> [u8; 32] {
    sha256_of_json(&provider_export_packet_value(records))
}
pub fn provider_export_packet_contains_id(
    records: &[ProviderProjectionRecord<'_>],
    wanted: &str,
) -> bool {
    records
        .iter()
        .any(|record| record.exportable && record.id == wanted)
}

pub fn provider_context_hashes(input: &ProviderProjectionInput<'_>) -> ProviderContextHashes {
    ProviderContextHashes {
        projected_packet_hash: provider_projection_packet_hash(input),
        exported_field_list_hash: projection_field_list_hash(
            "raios.provider_minimal.exported_fields.canonical.v0",
            PROVIDER_MINIMAL_INCLUDED_FIELDS,
        ),
        omitted_field_list_hash: projection_field_list_hash(
            "raios.provider_minimal.omitted_fields.canonical.v0",
            PROVIDER_MINIMAL_OMITTED_FIELDS,
        ),
        redaction_policy_hash: provider_redaction_policy_hash(),
        field_classification_hash: provider_field_classification_hash(),
        token_budget_hash: provider_token_budget_hash(),
    }
}

pub fn provider_projection_packet_hash(input: &ProviderProjectionInput<'_>) -> [u8; 32] {
    let mut h = EvidenceHash::new("raios.provider_minimal.packet.canonical.v0");
    h.field("schema", "raios.agent_context.v0");
    h.field("purpose", "current_boot_provider_context");
    h.field("profile", "provider_minimal");
    h.field("scope", "current_boot");
    h.field("budget.target_tokens", "2000");
    h.field("budget.estimated_tokens", "900");
    h.array(
        "authority_order",
        &["current_snapshot", "decision", "service_state", "summary"],
    );
    h.array("included.identity", &["mem.fact.identity.stage0"]);
    h.array("included.policy", &["adr.0001", "adr.0004"]);
    h.array(
        "included.current",
        &[
            "snapshot.current.provider_minimal",
            "capabilities.current_boot",
            "service.inventory.current",
            "problem.list.current",
        ],
    );
    h.field("current.os.name", "raiOS");
    h.field("current.os.product", "raiOS");
    h.field("current.os.stage", "stage-0");
    for field in input.status {
        h.field(field.field, field.value)
    }
    for field in input.provider {
        h.field(field.field, field.value)
    }
    for service in input.services {
        h.field("current.services[]", service)
    }
    for capability in input.capabilities {
        h.field("current.capabilities[]", capability)
    }
    for (idx, problem) in input.problems.iter().enumerate() {
        h.field(problem.field, problem.value);
        if (idx + 1) % 3 == 0 {
            h.separator();
        }
    }
    for record in input.records {
        if record.exportable {
            h.field("records[].id", record.id);
            h.field("records[].kind", record.kind);
            h.field("records[].authority", record.authority);
            h.field("records[].classification", record.classification);
            h.field("records[].summary", record.predicate);
            h.separator();
        }
    }
    h.array(
        "packet.omitted",
        &[
            "system.snapshot.raw",
            "system.boot_log.raw",
            "current.recovery_lifeline_status",
            "recovery.lifeline.status.current_boot",
            "unclassified.memory_context",
        ],
    );
    h.finish()
}

fn f<'a>(key: &'a str, value: Value<'a>) -> Field<'a> {
    Field::new(key, value)
}
struct EvidenceHash(Sha256);
impl EvidenceHash {
    fn new(domain: &'static str) -> Self {
        let mut h = Self(Sha256::new());
        h.field("domain", domain);
        h
    }
    fn field(&mut self, name: &str, value: &str) {
        self.0.update(name.as_bytes());
        self.0.update(b"=");
        self.0.update(value.as_bytes());
        self.0.update(b"\n")
    }
    fn array(&mut self, name: &str, values: &[&str]) {
        for value in values {
            self.field(name, value)
        }
        self.separator()
    }
    fn separator(&mut self) {
        self.0.update(b"--\n")
    }
    fn finish(self) -> [u8; 32] {
        let d = self.0.finalize();
        let mut out = [0; 32];
        out.copy_from_slice(&d);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projection_order_and_classification_are_fixed() {
        assert_eq!(PROVIDER_MINIMAL_INCLUDED_FIELDS[0].field, "schema");
        assert_eq!(
            PROVIDER_MINIMAL_INCLUDED_FIELDS[33].field,
            "records[].summary"
        );
        assert_eq!(PROVIDER_MINIMAL_OMITTED_FIELDS[0].field, "source_schemas");
        assert_eq!(
            PROVIDER_MINIMAL_OMITTED_FIELDS[18].field,
            "records[].source"
        );
        assert!(PROVIDER_MINIMAL_OMITTED_FIELDS
            .iter()
            .all(|f| f.action == "omit"));
    }
    #[test]
    fn export_method_profile_and_trust_are_fail_closed() {
        assert!(provider_context_export_method(
            "provider.export_context provider_minimal"
        ));
        assert_eq!(
            provider_context_export_profile("provider.context_export provider_minimal"),
            "provider_minimal"
        );
        assert_eq!(
            provider_context_export_profile("provider.context_export secret"),
            "unsupported"
        );
        assert!(provider_trust_positive("pinned_spki_verified"));
        assert!(!provider_trust_positive("pin_config_missing"));
        assert_eq!(
            provider_context_block_reason("pin_config_missing"),
            "provider_trust_not_positive"
        );
    }
    #[test]
    fn export_packet_contains_only_public_records() {
        let records = [
            ProviderProjectionRecord {
                id: "public",
                kind: "fact",
                entity: "e",
                predicate: "p",
                classification: "public",
                authority: "current_snapshot",
                exportable: true,
            },
            ProviderProjectionRecord {
                id: "local",
                kind: "fact",
                entity: "e",
                predicate: "p",
                classification: "local_only",
                authority: "current_snapshot",
                exportable: false,
            },
        ];
        let value = provider_export_packet_value(&records);
        assert_eq!(provider_export_packet_contains_id(&records, "public"), true);
        assert!(!provider_export_packet_contains_id(&records, "local"));
        assert_eq!(
            sha256_of_json(&value),
            provider_export_packet_hash(&records)
        );
    }

    #[test]
    fn projection_packet_problem_separators_match_hand_derived_hashes() {
        fn input<'a>(problems: &'a [ProviderProjectionField<'a>]) -> ProviderProjectionInput<'a> {
            ProviderProjectionInput {
                status: &[],
                provider: &[],
                services: &[],
                capabilities: &[],
                problems,
                records: &[],
            }
        }

        let empty = input(&[]);
        assert_eq!(
            provider_projection_packet_hash(&empty),
            [
                0x87, 0x43, 0x09, 0x7c, 0x18, 0xb7, 0x66, 0x9e, 0x1b, 0x49, 0x36, 0xb7, 0x2a, 0x32,
                0x96, 0xf7, 0xaa, 0x88, 0x1c, 0x44, 0x90, 0x61, 0x89, 0x02, 0x51, 0x64, 0xa1, 0x66,
                0xdf, 0xd8, 0xc9, 0x48,
            ]
        );

        let one_problem = [
            ProviderProjectionField {
                field: "current.problems[].id",
                value: "p1",
            },
            ProviderProjectionField {
                field: "current.problems[].severity",
                value: "warning",
            },
            ProviderProjectionField {
                field: "current.problems[].summary",
                value: "problem 1",
            },
        ];
        assert_eq!(
            provider_projection_packet_hash(&input(&one_problem)),
            [
                0x9a, 0x09, 0x35, 0xa6, 0xe6, 0xf7, 0x13, 0xcf, 0x4a, 0x5e, 0x0c, 0x84, 0x80, 0x53,
                0x55, 0x58, 0xc3, 0xd3, 0xb2, 0x09, 0xec, 0xef, 0xb9, 0x9d, 0x7d, 0xfd, 0x4b, 0x0d,
                0xd6, 0xab, 0xc4, 0x8d,
            ]
        );

        let two_problems = [
            one_problem[0],
            one_problem[1],
            one_problem[2],
            ProviderProjectionField {
                field: "current.problems[].id",
                value: "p2",
            },
            ProviderProjectionField {
                field: "current.problems[].severity",
                value: "warning",
            },
            ProviderProjectionField {
                field: "current.problems[].summary",
                value: "problem 2",
            },
        ];
        // Hand-derived from the canonical stream, including "--\n" after each problem triplet.
        assert_eq!(
            provider_projection_packet_hash(&input(&two_problems)),
            [
                0x3c, 0xc9, 0x28, 0x8c, 0x6c, 0x15, 0x2b, 0xa7, 0xd0, 0x5f, 0x63, 0x00, 0xf0, 0xcb,
                0x92, 0xfd, 0x79, 0x77, 0xa2, 0x69, 0xf4, 0x1c, 0x4e, 0x4b, 0xf6, 0xda, 0xf3, 0x9d,
                0x12, 0x2c, 0xde, 0x6d,
            ]
        );
    }
}
