//! `raios.memory_record.v0` — the typed durable-memory record model (M9A-1).
//!
//! This is the raios-core "split 1a" for M9 durable memory: one canonical,
//! host-tested serialization + hash for durable memory records that every later
//! write path reuses instead of inventing emit code. It is built ENTIRELY on the
//! existing `record.rs` Value/Field model and its single serializer+hasher
//! (`write_json` / `sha256_of_json` / `Value::Sha256`); this module hand-rolls no
//! JSON and no hex/hash formatting. Nothing calls it yet — M9A-2 wires the kernel.
//!
//! Field set is fixed by ADR 0004 (System Memory And Agent Context): schema, id,
//! kind, entity, predicate, value, classification, authority, boot_id, sequence,
//! source{method, record_id}, evidence[], tags[], supersedes[],
//! created_at{clock:"boot_relative", ticks}. There is NO wall-clock time —
//! `created_at` is boot-relative ticks; M10 owns trusted time.
//!
//! The constructor is fail-closed (ADR 0004 / M9 map decisions 2-4):
//!   - `secret` classification is NEVER durable — rejected with reason
//!     `secret_never_durable_until_sealed_secret_design`. A secret plaintext is
//!     structurally un-constructable: [`Classification`] has no `Secret` variant.
//!   - unknown/unrecognised classification input defaults to `local_only`, never
//!     silently `public`.
//!   - an unknown `kind` string is a typed error; only the eight authority-bearing
//!     kinds parse. Invalid kinds are structurally un-representable in
//!     [`MemoryKind`].
//!   - an `observation` must name its service (`entity`) and its source snapshot
//!     (`source`); an empty entity or source is a typed error.
//!   - supersede-not-overwrite: constructing a superseding record mutates nothing;
//!     it only carries `supersedes: [old_id]` links.

use alloc::{vec, vec::Vec};

use crate::record::{sha256_of_json, Field, Value};

/// The one durable memory record schema id. Every record renders this verbatim.
pub const SCHEMA: &str = "raios.memory_record.v0";

/// `created_at.clock` is always boot-relative in M9 — trusted wall-clock is M10.
pub const CREATED_AT_CLOCK: &str = "boot_relative";

/// Fail-closed constructor errors. Every rejection is a typed value, never a panic
/// or unwrap on caller input; `reason()` maps each to a stable `&'static str`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryRecordError {
    /// `secret` classification was requested for a durable record. Secrets are
    /// never durable (ADR 0004); only state markers may live inside non-secret
    /// records. The plaintext record is not produced.
    SecretNeverDurable,
    /// The `kind` string is not one of the eight authority-bearing kinds.
    UnknownKind,
    /// An `observation` did not name the service it observed (`entity` empty).
    ObservationMissingEntity,
    /// An `observation` did not name its source snapshot (`source` incomplete).
    ObservationMissingSource,
}

impl MemoryRecordError {
    /// Stable machine reason string for this rejection.
    pub const fn reason(self) -> &'static str {
        match self {
            MemoryRecordError::SecretNeverDurable => {
                "secret_never_durable_until_sealed_secret_design"
            }
            MemoryRecordError::UnknownKind => "memory_record_kind_out_of_scope",
            MemoryRecordError::ObservationMissingEntity => "observation_missing_entity",
            MemoryRecordError::ObservationMissingSource => "observation_missing_source",
        }
    }
}

/// The authority-bearing durable record kinds (M9 map decision 1). No other kind
/// string can be represented, so an unknown kind cannot silently become durable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryKind {
    CapabilityGrant,
    CapabilityDenial,
    PromotionTxRef,
    RollbackTxRef,
    Decision,
    Problem,
    Observation,
    ExportAudit,
}

impl MemoryKind {
    /// Parses a wire `kind` string; an unrecognised kind is a typed error.
    pub fn parse(kind: &str) -> Result<Self, MemoryRecordError> {
        Ok(match kind {
            "capability_grant" => MemoryKind::CapabilityGrant,
            "capability_denial" => MemoryKind::CapabilityDenial,
            "promotion_tx_ref" => MemoryKind::PromotionTxRef,
            "rollback_tx_ref" => MemoryKind::RollbackTxRef,
            "decision" => MemoryKind::Decision,
            "problem" => MemoryKind::Problem,
            "observation" => MemoryKind::Observation,
            "export_audit" => MemoryKind::ExportAudit,
            _ => return Err(MemoryRecordError::UnknownKind),
        })
    }

    /// The canonical wire string for this kind.
    pub const fn as_str(self) -> &'static str {
        match self {
            MemoryKind::CapabilityGrant => "capability_grant",
            MemoryKind::CapabilityDenial => "capability_denial",
            MemoryKind::PromotionTxRef => "promotion_tx_ref",
            MemoryKind::RollbackTxRef => "rollback_tx_ref",
            MemoryKind::Decision => "decision",
            MemoryKind::Problem => "problem",
            MemoryKind::Observation => "observation",
            MemoryKind::ExportAudit => "export_audit",
        }
    }
}

/// Durable classification (ADR 0004 Memory Classes). There is deliberately no
/// `Secret` variant: a secret plaintext record is structurally un-constructable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Classification {
    Public,
    LocalOnly,
}

impl Classification {
    /// Fail-closed classification parse: `secret` is REJECTED (never durable),
    /// and any unknown/unrecognised input defaults to `local_only` — never
    /// silently `public`.
    pub fn parse(classification: &str) -> Result<Self, MemoryRecordError> {
        match classification {
            "public" => Ok(Classification::Public),
            "local_only" => Ok(Classification::LocalOnly),
            "secret" => Err(MemoryRecordError::SecretNeverDurable),
            _ => Ok(Classification::LocalOnly),
        }
    }

    /// The canonical wire string for this classification.
    pub const fn as_str(self) -> &'static str {
        match self {
            Classification::Public => "public",
            Classification::LocalOnly => "local_only",
        }
    }
}

/// The `source` sub-object: which method produced the record and which prior
/// record/snapshot it derives from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemorySource<'a> {
    pub method: &'a str,
    pub record_id: &'a str,
}

impl<'a> MemorySource<'a> {
    pub const fn new(method: &'a str, record_id: &'a str) -> Self {
        Self { method, record_id }
    }

    /// True when neither the method nor the record id names anything.
    pub fn is_empty(&self) -> bool {
        self.method.is_empty() && self.record_id.is_empty()
    }

    /// True when the source fails to fully name method AND source snapshot.
    fn is_incomplete(&self) -> bool {
        self.method.is_empty() || self.record_id.is_empty()
    }
}

/// Unvalidated input to [`MemoryRecord::new`]. `kind` and `classification` arrive
/// as raw strings so the constructor can enforce the fail-closed rules; everything
/// else is already typed.
pub struct MemoryRecordInput<'a> {
    pub id: &'a str,
    pub kind: &'a str,
    pub entity: &'a str,
    pub predicate: &'a str,
    pub value: Value<'a>,
    pub classification: &'a str,
    pub authority: &'a str,
    pub boot_id: &'a str,
    pub sequence: u64,
    pub source: MemorySource<'a>,
    pub evidence: Vec<&'a str>,
    pub tags: Vec<&'a str>,
    pub supersedes: Vec<&'a str>,
    pub created_at_ticks: u64,
}

/// A validated `raios.memory_record.v0`. Construct via [`MemoryRecord::new`]; the
/// invariant-bearing fields (`kind`, `classification`) are enums that cannot hold
/// an out-of-scope kind or a secret classification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryRecord<'a> {
    pub id: &'a str,
    pub kind: MemoryKind,
    pub entity: &'a str,
    pub predicate: &'a str,
    pub value: Value<'a>,
    pub classification: Classification,
    pub authority: &'a str,
    pub boot_id: &'a str,
    pub sequence: u64,
    pub source: MemorySource<'a>,
    pub evidence: Vec<&'a str>,
    pub tags: Vec<&'a str>,
    pub supersedes: Vec<&'a str>,
    pub created_at_ticks: u64,
}

impl<'a> MemoryRecord<'a> {
    /// Fail-closed constructor. Returns `Err` (never panics) on any bad input:
    /// secret classification, unknown kind, or an observation missing its entity
    /// or source. Unknown classification input defaults to `local_only`.
    pub fn new(input: MemoryRecordInput<'a>) -> Result<Self, MemoryRecordError> {
        let kind = MemoryKind::parse(input.kind)?;
        let classification = Classification::parse(input.classification)?;

        if kind == MemoryKind::Observation {
            if input.entity.is_empty() {
                return Err(MemoryRecordError::ObservationMissingEntity);
            }
            if input.source.is_incomplete() {
                return Err(MemoryRecordError::ObservationMissingSource);
            }
        }

        Ok(Self {
            id: input.id,
            kind,
            entity: input.entity,
            predicate: input.predicate,
            value: input.value,
            classification,
            authority: input.authority,
            boot_id: input.boot_id,
            sequence: input.sequence,
            source: input.source,
            evidence: input.evidence,
            tags: input.tags,
            supersedes: input.supersedes,
            created_at_ticks: input.created_at_ticks,
        })
    }

    /// The record's fields in canonical order, ready for `record.rs::write_json`.
    pub fn fields(&self) -> Vec<Field<'_>> {
        vec![
            Field::new("schema", Value::Str(SCHEMA)),
            Field::new("id", Value::Str(self.id)),
            Field::new("kind", Value::Str(self.kind.as_str())),
            Field::new("entity", Value::Str(self.entity)),
            Field::new("predicate", Value::Str(self.predicate)),
            Field::new("value", self.value.clone()),
            Field::new("classification", Value::Str(self.classification.as_str())),
            Field::new("authority", Value::Str(self.authority)),
            Field::new("boot_id", Value::Str(self.boot_id)),
            Field::new("sequence", Value::U64(self.sequence)),
            Field::new(
                "source",
                Value::Object(vec![
                    Field::new("method", Value::Str(self.source.method)),
                    Field::new("record_id", Value::Str(self.source.record_id)),
                ]),
            ),
            Field::new("evidence", str_array(&self.evidence)),
            Field::new("tags", str_array(&self.tags)),
            Field::new("supersedes", str_array(&self.supersedes)),
            Field::new(
                "created_at",
                Value::Object(vec![
                    Field::new("clock", Value::Str(CREATED_AT_CLOCK)),
                    Field::new("ticks", Value::U64(self.created_at_ticks)),
                ]),
            ),
        ]
    }

    /// Renders the whole record through the single record-model serializer.
    pub fn to_record_value(&self) -> Value<'_> {
        Value::Object(self.fields())
    }

    /// The SHA-256 of the canonically rendered record, via the single hasher.
    pub fn record_sha256(&self) -> [u8; 32] {
        sha256_of_json(&self.to_record_value())
    }
}

/// Renders a list of string ids as a record-model array of `Value::Str`.
fn str_array<'a>(items: &[&'a str]) -> Value<'a> {
    let mut values = Vec::with_capacity(items.len());
    for &item in items {
        values.push(Value::Str(item));
    }
    Value::Array(values)
}

#[cfg(test)]
mod tests {
    use super::{
        Classification, MemoryKind, MemoryRecord, MemoryRecordError, MemoryRecordInput,
        MemorySource, CREATED_AT_CLOCK, SCHEMA,
    };
    use crate::record::{write_json, Field, Value};
    use crate::sha256_hex;

    /// A fixed digest reused for the sample record's inline evidence hash.
    const SAMPLE_DIGEST: [u8; 32] = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd,
        0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
        0xcd, 0xef,
    ];

    /// PINNED sha256 of the fixed sample record's canonical rendering. M9A-2 can
    /// needle this exact hex against the on-disk frame it writes.
    const FIXED_SAMPLE_SHA256_HEX: &str =
        "4ab57d937747c8f1d4c9b44375f57fdf5654d512ac2ae206b32d68f9f5413928";

    fn render_string(value: &Value<'_>) -> std::string::String {
        let mut out = std::vec::Vec::new();
        write_json(value, &mut out, 0);
        std::string::String::from_utf8(out).expect("record renders as ascii json")
    }

    fn hex_string(digest: &[u8; 32]) -> std::string::String {
        std::string::String::from_utf8(sha256_hex(digest).to_vec()).expect("hex is ascii")
    }

    /// The canonical fixed sample: an agent observation of a denied provider
    /// trust check, carrying an inline evidence hash (a `Value::Sha256`).
    fn fixed_sample_record() -> MemoryRecord<'static> {
        MemoryRecord::new(MemoryRecordInput {
            id: "mem.event.00000042",
            kind: "observation",
            entity: "svc.provider.openai_direct",
            predicate: "provider_trust_denied",
            value: Value::InlineObject(vec![
                Field::new("reason", Value::Str("pin_config_missing")),
                Field::new("evidence_hash", Value::Sha256(SAMPLE_DIGEST)),
            ]),
            classification: "public",
            authority: "event",
            boot_id: "boot:0000000000000001",
            sequence: 42,
            source: MemorySource::new("system.snapshot", "snapshot:current_boot.00000007"),
            evidence: vec!["problem:provider.tls_pin_config_missing"],
            tags: vec!["provider", "tls", "trust"],
            supersedes: vec![],
            created_at_ticks: 12345,
        })
        .expect("fixed sample record must construct")
    }

    #[test]
    fn fixed_sample_record_hash_is_pinned() {
        let record = fixed_sample_record();
        let rendered = render_string(&record.to_record_value());

        // Stable shape: schema, boot-relative clock, and no wall-clock time.
        assert!(rendered.contains("\"schema\": \"raios.memory_record.v0\""));
        assert!(rendered.contains("\"clock\": \"boot_relative\""));
        assert_eq!(SCHEMA, "raios.memory_record.v0");
        assert_eq!(CREATED_AT_CLOCK, "boot_relative");

        // Pinned hash: the serialize+hash path is stable across runs.
        assert_eq!(hex_string(&record.record_sha256()), FIXED_SAMPLE_SHA256_HEX);
    }

    #[test]
    fn sha256_field_renders_kernel_form() {
        let record = fixed_sample_record();
        let rendered = render_string(&record.to_record_value());
        let expected = std::format!("sha256:{}", hex_string(&SAMPLE_DIGEST));
        assert!(
            rendered.contains(&expected),
            "record must render Value::Sha256 as sha256:<64hex>"
        );
    }

    #[test]
    fn secret_classification_is_never_durable() {
        let result = MemoryRecord::new(MemoryRecordInput {
            id: "mem.secret.1",
            kind: "capability_grant",
            entity: "svc.secret_holder",
            predicate: "api_key",
            value: Value::Str("SHOULD-NOT-BE-DURABLE"),
            classification: "secret",
            authority: "core_ledger",
            boot_id: "boot:1",
            sequence: 1,
            source: MemorySource::new("system.snapshot", "snapshot:1"),
            evidence: vec![],
            tags: vec![],
            supersedes: vec![],
            created_at_ticks: 1,
        });
        assert_eq!(result, Err(MemoryRecordError::SecretNeverDurable));
        assert_eq!(
            result.unwrap_err().reason(),
            "secret_never_durable_until_sealed_secret_design"
        );
    }

    #[test]
    fn unknown_kind_is_rejected() {
        let result = MemoryRecord::new(MemoryRecordInput {
            id: "mem.x.1",
            kind: "chat_history",
            entity: "svc.x",
            predicate: "p",
            value: Value::Null,
            classification: "local_only",
            authority: "event",
            boot_id: "boot:1",
            sequence: 1,
            source: MemorySource::new("m", "r"),
            evidence: vec![],
            tags: vec![],
            supersedes: vec![],
            created_at_ticks: 1,
        });
        assert_eq!(result, Err(MemoryRecordError::UnknownKind));
        assert_eq!(
            result.unwrap_err().reason(),
            "memory_record_kind_out_of_scope"
        );
    }

    #[test]
    fn all_authority_bearing_kinds_parse() {
        for kind in [
            "capability_grant",
            "capability_denial",
            "promotion_tx_ref",
            "rollback_tx_ref",
            "decision",
            "problem",
            "observation",
            "export_audit",
        ] {
            assert_eq!(MemoryKind::parse(kind).unwrap().as_str(), kind);
        }
    }

    #[test]
    fn observation_requires_entity_and_source() {
        let missing_entity = MemoryRecord::new(MemoryRecordInput {
            id: "mem.obs.1",
            kind: "observation",
            entity: "",
            predicate: "p",
            value: Value::Null,
            classification: "local_only",
            authority: "event",
            boot_id: "boot:1",
            sequence: 1,
            source: MemorySource::new("system.snapshot", "snapshot:1"),
            evidence: vec![],
            tags: vec![],
            supersedes: vec![],
            created_at_ticks: 1,
        });
        assert_eq!(
            missing_entity,
            Err(MemoryRecordError::ObservationMissingEntity)
        );

        let missing_source = MemoryRecord::new(MemoryRecordInput {
            id: "mem.obs.2",
            kind: "observation",
            entity: "svc.x",
            predicate: "p",
            value: Value::Null,
            classification: "local_only",
            authority: "event",
            boot_id: "boot:1",
            sequence: 1,
            source: MemorySource::new("", ""),
            evidence: vec![],
            tags: vec![],
            supersedes: vec![],
            created_at_ticks: 1,
        });
        assert_eq!(
            missing_source,
            Err(MemoryRecordError::ObservationMissingSource)
        );
    }

    #[test]
    fn unknown_classification_defaults_to_local_only_never_public() {
        let record = MemoryRecord::new(MemoryRecordInput {
            id: "mem.dec.1",
            kind: "decision",
            entity: "adr.0004",
            predicate: "classified",
            value: Value::Null,
            classification: "banana",
            authority: "decision",
            boot_id: "boot:1",
            sequence: 1,
            source: MemorySource::new("system.snapshot", "snapshot:1"),
            evidence: vec![],
            tags: vec![],
            supersedes: vec![],
            created_at_ticks: 1,
        })
        .expect("unknown classification must not fail, it defaults");
        assert_eq!(record.classification, Classification::LocalOnly);

        let rendered = render_string(&record.to_record_value());
        assert!(rendered.contains("\"classification\": \"local_only\""));
        assert!(!rendered.contains("\"classification\": \"public\""));
    }

    #[test]
    fn supersedes_round_trips_and_construction_mutates_nothing() {
        let original = MemoryRecord::new(MemoryRecordInput {
            id: "mem.problem.open.1",
            kind: "problem",
            entity: "svc.provider.openai_direct",
            predicate: "provider_trust_denied",
            value: Value::Null,
            classification: "local_only",
            authority: "event",
            boot_id: "boot:1",
            sequence: 1,
            source: MemorySource::new("system.snapshot", "snapshot:1"),
            evidence: vec![],
            tags: vec![],
            supersedes: vec![],
            created_at_ticks: 10,
        })
        .expect("open problem record must construct");
        let original_before = render_string(&original.to_record_value());

        let superseding = MemoryRecord::new(MemoryRecordInput {
            id: "mem.problem.resolved.1",
            kind: "problem",
            entity: "svc.provider.openai_direct",
            predicate: "provider_trust_resolved",
            value: Value::Null,
            classification: "local_only",
            authority: "event",
            boot_id: "boot:1",
            sequence: 2,
            source: MemorySource::new("system.snapshot", "snapshot:2"),
            evidence: vec!["problem:provider.tls_pin_config_missing"],
            tags: vec![],
            supersedes: vec!["mem.problem.open.1"],
            created_at_ticks: 20,
        })
        .expect("superseding record must construct");

        // The link round-trips in the rendered value.
        let rendered = render_string(&superseding.to_record_value());
        assert!(rendered.contains("\"supersedes\": [\r\n    \"mem.problem.open.1\"\r\n  ]"));

        // Supersede-not-overwrite: the original is untouched by constructing the
        // superseding record.
        assert!(original.supersedes.is_empty());
        assert_eq!(render_string(&original.to_record_value()), original_before);
    }
}
