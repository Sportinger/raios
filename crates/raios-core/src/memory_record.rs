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
use core::str;

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
    /// A `decision` did not name the entity it decided about (`entity` empty).
    DecisionMissingEntity,
    /// A `decision` did not name its source snapshot (`source` incomplete).
    DecisionMissingSource,
    /// A `problem` did not name the entity it concerns (`entity` empty).
    ProblemMissingEntity,
    /// A `problem` did not carry a status (`predicate` empty).
    ProblemMissingStatus,
    /// One of the 5 audit/authority kinds was authored as a superseding record.
    /// Audit kinds may never supersede — the reclog audit trail must never be
    /// hideable by a future broker resolving a `supersedes` link.
    AuditKindMayNotSupersede,
    /// `supersedes` named more ids than [`MAX_SUPERSEDES_PER_RECORD`] allows.
    SupersedesListTooLong,
    /// `supersedes` named this record's own id.
    SelfSupersede,
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
            MemoryRecordError::DecisionMissingEntity => "decision_missing_entity",
            MemoryRecordError::DecisionMissingSource => "decision_missing_source",
            MemoryRecordError::ProblemMissingEntity => "problem_missing_entity",
            MemoryRecordError::ProblemMissingStatus => "problem_missing_status",
            MemoryRecordError::AuditKindMayNotSupersede => "audit_kind_may_not_supersede",
            MemoryRecordError::SupersedesListTooLong => "supersedes_list_too_long",
            MemoryRecordError::SelfSupersede => "supersede_self_reference",
        }
    }
}

/// Maximum number of ids a single record's `supersedes` list may name. Bounds
/// the write-side blast radius of a supersede link (M9A-3).
pub const MAX_SUPERSEDES_PER_RECORD: usize = 8;

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

    /// True for the 5 audit/authority kinds that may never be authored as a
    /// superseding record (M9A-3 write-side supersede-hazard closure).
    pub const fn is_audit(self) -> bool {
        matches!(
            self,
            MemoryKind::CapabilityGrant
                | MemoryKind::CapabilityDenial
                | MemoryKind::PromotionTxRef
                | MemoryKind::RollbackTxRef
                | MemoryKind::ExportAudit
        )
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

        if kind == MemoryKind::Decision {
            if input.entity.is_empty() {
                return Err(MemoryRecordError::DecisionMissingEntity);
            }
            if input.source.is_incomplete() {
                return Err(MemoryRecordError::DecisionMissingSource);
            }
        }

        if kind == MemoryKind::Problem {
            if input.entity.is_empty() {
                return Err(MemoryRecordError::ProblemMissingEntity);
            }
            if input.predicate.is_empty() {
                return Err(MemoryRecordError::ProblemMissingStatus);
            }
        }

        if kind.is_audit() && !input.supersedes.is_empty() {
            return Err(MemoryRecordError::AuditKindMayNotSupersede);
        }
        if input.supersedes.len() > MAX_SUPERSEDES_PER_RECORD {
            return Err(MemoryRecordError::SupersedesListTooLong);
        }
        if input.supersedes.iter().any(|&s| s == input.id) {
            return Err(MemoryRecordError::SelfSupersede);
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

// ---------------------------------------------------------------------------
// M9C-1a: fail-closed reparser (durable payload bytes -> typed metadata VIEW).
//
// This is the FIRST decode of on-disk payload bytes back into memory metadata.
// It is a bounded, WHITESPACE-TOLERANT, canonical tokenizer — NOT a general JSON
// parser and NOT a byte-exact indentation matcher. It walks the exact canonical
// field key SEQUENCE written by `to_record_value()` + `record::write_json`,
// validates the scalar values, and STRUCTURALLY SKIPS the `value` sub-tree (its
// raw content is deliberately never decoded, so arbitrary escaped agent text can
// never reach the broker through this path).
//
// It is all-or-nothing: any deviation from the canonical shape returns a typed
// `Err` (a dropped frame) — it never panics/unwraps on payload bytes.
// ---------------------------------------------------------------------------

/// Maximum nesting depth the fail-closed `value`-subtree walker will descend
/// before rejecting. The write path never nests a memory `value` tree deeply; a
/// deeper tree is treated as corrupt input rather than walked unboundedly.
const MAX_VALUE_DEPTH: usize = 32;

/// The canonical field-key order emitted by [`MemoryRecord::fields`]. The
/// reparser expects exactly these keys, in exactly this order.
const CANONICAL_FIELDS: [&str; 15] = [
    "schema",
    "id",
    "kind",
    "entity",
    "predicate",
    "value",
    "classification",
    "authority",
    "boot_id",
    "sequence",
    "source",
    "evidence",
    "tags",
    "supersedes",
    "created_at",
];

/// A typed, read-only metadata view over a durable `raios.memory_record.v0`
/// payload. It carries every authority-bearing metadata field EXCEPT `value`:
/// the `value` sub-tree is skipped structurally and never decoded, which shrinks
/// the decode surface and keeps raw agent content out of the context broker. All
/// string fields borrow directly from the payload bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryRecordView<'a> {
    pub id: &'a str,
    pub kind: MemoryKind,
    pub entity: &'a str,
    pub predicate: &'a str,
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

/// Fail-closed reparse errors. Every deviation from the canonical on-disk shape
/// maps to one of these; `reason()` gives a stable, pairwise-unique string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryRecordParseError {
    /// A payload byte was outside the ASCII range `write_json` ever emits.
    NotAscii,
    /// The `schema` field was not the one durable memory schema id.
    SchemaMismatch,
    /// The `kind` string is not one of the eight authority-bearing kinds.
    UnknownKind,
    /// The `classification` field was `secret` — never durable.
    SecretClassification,
    /// A canonical field was absent, or a structural token was missing/malformed.
    MissingField,
    /// A canonical field key appeared out of its canonical order.
    FieldOrder,
    /// A key that is not a canonical field appeared where a field was expected.
    ExtraField,
    /// Bytes remained after the final closing brace (only whitespace is allowed).
    TrailingBytes,
    /// The skipped `value` sub-tree was unbalanced, unterminated, or too deep.
    ValueUnbalanced,
    /// A numeric field was empty, non-digit, non-canonical, or overflowed `u64`.
    BadNumber,
    /// A metadata string was unterminated or carried a backslash escape.
    BadString,
    /// An `observation` did not name the service it observed (`entity` empty).
    ObservationMissingEntity,
    /// An `observation` did not name its source snapshot (`source` incomplete).
    ObservationMissingSource,
    /// A `decision` did not name the entity it decided about (`entity` empty).
    DecisionMissingEntity,
    /// A `decision` did not name its source snapshot (`source` incomplete).
    DecisionMissingSource,
    /// A `problem` did not name the entity it concerns (`entity` empty).
    ProblemMissingEntity,
    /// A `problem` did not carry a status (`predicate` empty).
    ProblemMissingStatus,
    /// `supersedes` named more ids than [`MAX_SUPERSEDES_PER_RECORD`] allows.
    SupersedesTooLong,
    /// `supersedes` named this record's own id.
    SelfSupersede,
}

impl MemoryRecordParseError {
    /// Stable machine reason string for this rejection. Pairwise-unique.
    pub const fn reason(self) -> &'static str {
        match self {
            MemoryRecordParseError::NotAscii => "payload_byte_not_ascii",
            MemoryRecordParseError::SchemaMismatch => "schema_id_mismatch",
            MemoryRecordParseError::UnknownKind => "kind_out_of_scope",
            MemoryRecordParseError::SecretClassification => "secret_classification_never_durable",
            MemoryRecordParseError::MissingField => "canonical_field_missing_or_malformed",
            MemoryRecordParseError::FieldOrder => "canonical_field_out_of_order",
            MemoryRecordParseError::ExtraField => "unknown_extra_field",
            MemoryRecordParseError::TrailingBytes => "trailing_bytes_after_record",
            MemoryRecordParseError::ValueUnbalanced => "value_subtree_unbalanced",
            MemoryRecordParseError::BadNumber => "numeric_field_not_canonical_u64",
            MemoryRecordParseError::BadString => "string_field_bad_or_unterminated",
            MemoryRecordParseError::ObservationMissingEntity => {
                "reparse_observation_missing_entity"
            }
            MemoryRecordParseError::ObservationMissingSource => {
                "reparse_observation_missing_source"
            }
            MemoryRecordParseError::DecisionMissingEntity => "reparse_decision_missing_entity",
            MemoryRecordParseError::DecisionMissingSource => "reparse_decision_missing_source",
            MemoryRecordParseError::ProblemMissingEntity => "reparse_problem_missing_entity",
            MemoryRecordParseError::ProblemMissingStatus => "reparse_problem_missing_status",
            MemoryRecordParseError::SupersedesTooLong => "reparse_supersedes_list_too_long",
            MemoryRecordParseError::SelfSupersede => "reparse_supersede_self_reference",
        }
    }
}

/// Position of `key` within the canonical field order, if it is a canonical key.
fn canonical_index(key: &str) -> Option<usize> {
    let mut idx = 0usize;
    while idx < CANONICAL_FIELDS.len() {
        if CANONICAL_FIELDS[idx] == key {
            return Some(idx);
        }
        idx += 1;
    }
    None
}

/// Classifies an unexpected key seen where `CANONICAL_FIELDS[expected]` was due.
fn classify_key(key: &str, expected: usize) -> MemoryRecordParseError {
    match canonical_index(key) {
        // Not a canonical key at all -> an extra/unknown key.
        None => MemoryRecordParseError::ExtraField,
        // A later canonical key appeared -> the expected field was skipped.
        Some(found) if found > expected => MemoryRecordParseError::MissingField,
        // An earlier/duplicate canonical key reappeared -> out of order.
        Some(_) => MemoryRecordParseError::FieldOrder,
    }
}

/// A bounded byte cursor over the payload. Never indexes out of bounds and never
/// panics on malformed input.
struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn bump(&mut self) {
        self.pos += 1;
    }

    fn at_end(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    /// Skips ASCII whitespace (space, CR, LF, TAB) between tokens.
    fn skip_ws(&mut self) {
        while let Some(byte) = self.peek() {
            if byte == b' ' || byte == b'\r' || byte == b'\n' || byte == b'\t' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn expect(
        &mut self,
        byte: u8,
        err: MemoryRecordParseError,
    ) -> Result<(), MemoryRecordParseError> {
        if self.peek() == Some(byte) {
            self.pos += 1;
            Ok(())
        } else {
            Err(err)
        }
    }

    /// Parses one `"..."` metadata-string token. Fail-closed choice (b): a
    /// backslash escape inside a metadata string is REJECTED (`BadString`) rather
    /// than unescaped, so every accepted metadata field borrows the raw inner
    /// slice with zero owned allocation. The `value` sub-tree — which is the only
    /// place arbitrary escaped agent text can appear — is skipped structurally, so
    /// its escapes never reach this path.
    fn parse_string(&mut self) -> Result<&'a str, MemoryRecordParseError> {
        self.skip_ws();
        if self.peek() != Some(b'"') {
            return Err(MemoryRecordParseError::MissingField);
        }
        self.pos += 1;
        let start = self.pos;
        loop {
            match self.peek() {
                None => return Err(MemoryRecordParseError::BadString),
                Some(b'\\') => return Err(MemoryRecordParseError::BadString),
                Some(b'"') => {
                    let slice = &self.bytes[start..self.pos];
                    self.pos += 1;
                    return str::from_utf8(slice).map_err(|_| MemoryRecordParseError::BadString);
                }
                Some(_) => self.pos += 1,
            }
        }
    }

    /// Parses a canonical, unpadded `u64` (no leading zeros, no sign, no plus).
    fn parse_u64(&mut self) -> Result<u64, MemoryRecordParseError> {
        self.skip_ws();
        let start = self.pos;
        let mut value: u64 = 0;
        while let Some(byte) = self.peek() {
            if byte.is_ascii_digit() {
                value = value
                    .checked_mul(10)
                    .and_then(|scaled| scaled.checked_add((byte - b'0') as u64))
                    .ok_or(MemoryRecordParseError::BadNumber)?;
                self.pos += 1;
            } else {
                break;
            }
        }
        let digits = &self.bytes[start..self.pos];
        if digits.is_empty() {
            return Err(MemoryRecordParseError::BadNumber);
        }
        // write_u64 never emits a leading zero except for the single digit "0".
        if digits.len() > 1 && digits[0] == b'0' {
            return Err(MemoryRecordParseError::BadNumber);
        }
        Ok(value)
    }

    /// Consumes exactly one balanced JSON value WITHOUT interpreting it (the
    /// `value` sub-tree). String-aware and depth-bounded; any imbalance,
    /// unterminated string, or over-depth is `ValueUnbalanced`.
    fn skip_value(&mut self) -> Result<(), MemoryRecordParseError> {
        self.skip_ws();
        match self.peek() {
            Some(b'"') => self.skip_string_structural(),
            Some(b'{') | Some(b'[') => self.skip_container(),
            Some(_) => self.skip_primitive(),
            None => Err(MemoryRecordParseError::ValueUnbalanced),
        }
    }

    /// Skips a `"..."` string honoring `\X` escapes (to find the true end); does
    /// NOT decode. Used only inside the skipped `value` sub-tree.
    fn skip_string_structural(&mut self) -> Result<(), MemoryRecordParseError> {
        self.pos += 1; // opening quote
        loop {
            match self.peek() {
                None => return Err(MemoryRecordParseError::ValueUnbalanced),
                Some(b'\\') => {
                    self.pos += 1;
                    if self.at_end() {
                        return Err(MemoryRecordParseError::ValueUnbalanced);
                    }
                    self.pos += 1; // skip the escaped byte, whatever it is
                }
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(());
                }
                Some(_) => self.pos += 1,
            }
        }
    }

    /// Skips a balanced `{...}` / `[...]` container with a depth bound and a
    /// matching-closer stack. String contents are skipped so their delimiters
    /// never miscount.
    fn skip_container(&mut self) -> Result<(), MemoryRecordParseError> {
        let mut closers = [0u8; MAX_VALUE_DEPTH];
        let mut depth = 0usize;
        loop {
            match self.peek() {
                None => return Err(MemoryRecordParseError::ValueUnbalanced),
                Some(open @ (b'{' | b'[')) => {
                    if depth >= MAX_VALUE_DEPTH {
                        return Err(MemoryRecordParseError::ValueUnbalanced);
                    }
                    closers[depth] = if open == b'{' { b'}' } else { b']' };
                    depth += 1;
                    self.pos += 1;
                }
                Some(close @ (b'}' | b']')) => {
                    if depth == 0 || closers[depth - 1] != close {
                        return Err(MemoryRecordParseError::ValueUnbalanced);
                    }
                    depth -= 1;
                    self.pos += 1;
                    if depth == 0 {
                        return Ok(());
                    }
                }
                Some(b'"') => self.skip_string_structural()?,
                Some(_) => self.pos += 1,
            }
        }
    }

    /// Skips a primitive value token (number/bool/null) up to the next structural
    /// delimiter. Requires at least one byte.
    fn skip_primitive(&mut self) -> Result<(), MemoryRecordParseError> {
        let start = self.pos;
        while let Some(byte) = self.peek() {
            if byte == b','
                || byte == b'}'
                || byte == b']'
                || byte == b' '
                || byte == b'\r'
                || byte == b'\n'
                || byte == b'\t'
            {
                break;
            }
            self.pos += 1;
        }
        if self.pos == start {
            return Err(MemoryRecordParseError::ValueUnbalanced);
        }
        Ok(())
    }

    /// Parses a canonical array of `"..."` strings, whitespace-tolerant. An empty
    /// array is `[]`. `cap` bounds the element count (used for `supersedes`).
    fn parse_string_array(
        &mut self,
        cap: Option<(usize, MemoryRecordParseError)>,
    ) -> Result<Vec<&'a str>, MemoryRecordParseError> {
        self.skip_ws();
        self.expect(b'[', MemoryRecordParseError::MissingField)?;
        self.skip_ws();
        let mut out = Vec::new();
        if self.peek() == Some(b']') {
            self.bump();
            return Ok(out);
        }
        loop {
            let item = self.parse_string()?;
            out.push(item);
            if let Some((max, too_long)) = cap {
                if out.len() > max {
                    return Err(too_long);
                }
            }
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.bump(),
                Some(b']') => {
                    self.bump();
                    return Ok(out);
                }
                _ => return Err(MemoryRecordParseError::ValueUnbalanced),
            }
        }
    }

    /// Parses the `source` object: exactly `{ "method": <str>, "record_id": <str> }`.
    fn parse_source(&mut self) -> Result<MemorySource<'a>, MemoryRecordParseError> {
        self.skip_ws();
        self.expect(b'{', MemoryRecordParseError::MissingField)?;
        let method = self.parse_member("method")?;
        self.skip_ws();
        self.expect(b',', MemoryRecordParseError::MissingField)?;
        let record_id = self.parse_member("record_id")?;
        self.skip_ws();
        self.expect(b'}', MemoryRecordParseError::MissingField)?;
        Ok(MemorySource::new(method, record_id))
    }

    /// Parses `"name": <str>` and returns the string value.
    fn parse_member(&mut self, name: &str) -> Result<&'a str, MemoryRecordParseError> {
        let key = self.parse_string()?;
        if key != name {
            return Err(MemoryRecordParseError::MissingField);
        }
        self.skip_ws();
        self.expect(b':', MemoryRecordParseError::MissingField)?;
        self.parse_string()
    }

    /// Parses the `created_at` object: `{ "clock": <CREATED_AT_CLOCK>, "ticks": <u64> }`,
    /// returning the ticks.
    fn parse_created_at(&mut self) -> Result<u64, MemoryRecordParseError> {
        self.skip_ws();
        self.expect(b'{', MemoryRecordParseError::MissingField)?;
        let clock = self.parse_member("clock")?;
        if clock != CREATED_AT_CLOCK {
            return Err(MemoryRecordParseError::MissingField);
        }
        self.skip_ws();
        self.expect(b',', MemoryRecordParseError::MissingField)?;
        let ticks_key = self.parse_string()?;
        if ticks_key != "ticks" {
            return Err(MemoryRecordParseError::MissingField);
        }
        self.skip_ws();
        self.expect(b':', MemoryRecordParseError::MissingField)?;
        let ticks = self.parse_u64()?;
        self.skip_ws();
        self.expect(b'}', MemoryRecordParseError::MissingField)?;
        Ok(ticks)
    }

    /// Consumes the comma separator that precedes every field after the first.
    fn expect_field_separator(&mut self) -> Result<(), MemoryRecordParseError> {
        self.skip_ws();
        match self.peek() {
            Some(b',') => {
                self.bump();
                Ok(())
            }
            // A closing brace here means the object ended before all canonical
            // fields were present.
            Some(b'}') => Err(MemoryRecordParseError::MissingField),
            _ => Err(MemoryRecordParseError::MissingField),
        }
    }

    /// Reads the canonical key at position `index` (with its trailing colon),
    /// classifying any deviation.
    fn expect_key(&mut self, index: usize) -> Result<(), MemoryRecordParseError> {
        let key = self.parse_string()?;
        if key != CANONICAL_FIELDS[index] {
            return Err(classify_key(key, index));
        }
        self.skip_ws();
        self.expect(b':', MemoryRecordParseError::MissingField)
    }
}

/// Fail-closed reparse of a durable `raios.memory_record.v0` payload back into a
/// typed [`MemoryRecordView`]. Returns `Err` (a dropped frame) for ANY deviation
/// from the canonical shape; never panics/unwraps on payload bytes. All-or-nothing.
pub fn parse(payload: &[u8]) -> Result<MemoryRecordView<'_>, MemoryRecordParseError> {
    // Deny-in-depth 0: write_json output is always ASCII (bytes >= 0x80 are
    // replaced with a space at emit time), so any high byte is corrupt input.
    if payload.iter().any(|&byte| byte >= 0x80) {
        return Err(MemoryRecordParseError::NotAscii);
    }

    let mut cursor = Cursor::new(payload);
    cursor.skip_ws();
    cursor.expect(b'{', MemoryRecordParseError::MissingField)?;

    // schema (LOAD-BEARING: the reclog is shared with non-memory frames — every
    // one hash-verifies but MUST drop here, because its schema id is not ours).
    cursor.expect_key(0)?;
    if cursor.parse_string()? != SCHEMA {
        return Err(MemoryRecordParseError::SchemaMismatch);
    }

    cursor.expect_field_separator()?;
    cursor.expect_key(1)?;
    let id = cursor.parse_string()?;

    cursor.expect_field_separator()?;
    cursor.expect_key(2)?;
    let kind = MemoryKind::parse(cursor.parse_string()?)
        .map_err(|_| MemoryRecordParseError::UnknownKind)?;

    cursor.expect_field_separator()?;
    cursor.expect_key(3)?;
    let entity = cursor.parse_string()?;

    cursor.expect_field_separator()?;
    cursor.expect_key(4)?;
    let predicate = cursor.parse_string()?;

    cursor.expect_field_separator()?;
    cursor.expect_key(5)?;
    // The `value` sub-tree is SKIPPED STRUCTURALLY and never decoded.
    cursor.skip_value()?;

    cursor.expect_field_separator()?;
    cursor.expect_key(6)?;
    let classification = Classification::parse(cursor.parse_string()?)
        .map_err(|_| MemoryRecordParseError::SecretClassification)?;

    cursor.expect_field_separator()?;
    cursor.expect_key(7)?;
    let authority = cursor.parse_string()?;

    cursor.expect_field_separator()?;
    cursor.expect_key(8)?;
    let boot_id = cursor.parse_string()?;

    cursor.expect_field_separator()?;
    cursor.expect_key(9)?;
    let sequence = cursor.parse_u64()?;

    cursor.expect_field_separator()?;
    cursor.expect_key(10)?;
    let source = cursor.parse_source()?;

    cursor.expect_field_separator()?;
    cursor.expect_key(11)?;
    let evidence = cursor.parse_string_array(None)?;

    cursor.expect_field_separator()?;
    cursor.expect_key(12)?;
    let tags = cursor.parse_string_array(None)?;

    cursor.expect_field_separator()?;
    cursor.expect_key(13)?;
    let supersedes = cursor.parse_string_array(Some((
        MAX_SUPERSEDES_PER_RECORD,
        MemoryRecordParseError::SupersedesTooLong,
    )))?;

    cursor.expect_field_separator()?;
    cursor.expect_key(14)?;
    let created_at_ticks = cursor.parse_created_at()?;

    // Final closing brace, then only trailing whitespace is allowed.
    cursor.skip_ws();
    match cursor.peek() {
        Some(b'}') => cursor.bump(),
        Some(b',') => return Err(MemoryRecordParseError::ExtraField),
        _ => return Err(MemoryRecordParseError::MissingField),
    }
    cursor.skip_ws();
    if !cursor.at_end() {
        return Err(MemoryRecordParseError::TrailingBytes);
    }

    // Deny-in-depth: the SAME invariants `MemoryRecord::new` enforces, minus the
    // write-only `audit_kind_may_not_supersede` rule (read-side R1 resolution
    // handles audit-supersede semantics, so it is deliberately NOT enforced here).
    match kind {
        MemoryKind::Observation => {
            if entity.is_empty() {
                return Err(MemoryRecordParseError::ObservationMissingEntity);
            }
            if source.method.is_empty() || source.record_id.is_empty() {
                return Err(MemoryRecordParseError::ObservationMissingSource);
            }
        }
        MemoryKind::Decision => {
            if entity.is_empty() {
                return Err(MemoryRecordParseError::DecisionMissingEntity);
            }
            if source.method.is_empty() || source.record_id.is_empty() {
                return Err(MemoryRecordParseError::DecisionMissingSource);
            }
        }
        MemoryKind::Problem => {
            if entity.is_empty() {
                return Err(MemoryRecordParseError::ProblemMissingEntity);
            }
            if predicate.is_empty() {
                return Err(MemoryRecordParseError::ProblemMissingStatus);
            }
        }
        _ => {}
    }
    if supersedes.iter().any(|&target| target == id) {
        return Err(MemoryRecordParseError::SelfSupersede);
    }

    Ok(MemoryRecordView {
        id,
        kind,
        entity,
        predicate,
        classification,
        authority,
        boot_id,
        sequence,
        source,
        evidence,
        tags,
        supersedes,
        created_at_ticks,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        parse, Classification, MemoryKind, MemoryRecord, MemoryRecordError, MemoryRecordInput,
        MemoryRecordParseError, MemoryRecordView, MemorySource, CREATED_AT_CLOCK,
        MAX_SUPERSEDES_PER_RECORD, SCHEMA,
    };
    use crate::record::{write_json, Field, Value};
    use crate::sha256_hex;
    use std::string::{String, ToString};
    use std::vec::Vec;

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

    /// A fully-valid `decision` input; individual fields are overridden per-test.
    fn valid_decision_input() -> MemoryRecordInput<'static> {
        MemoryRecordInput {
            id: "mem.decision.1",
            kind: "decision",
            entity: "adr.0004",
            predicate: "memory_is_typed_state",
            value: Value::Null,
            classification: "local_only",
            authority: "decision",
            boot_id: "boot:1",
            sequence: 1,
            source: MemorySource::new("system.snapshot", "snapshot:1"),
            evidence: vec![],
            tags: vec![],
            supersedes: vec![],
            created_at_ticks: 1,
        }
    }

    /// A fully-valid `problem` input; individual fields are overridden per-test.
    fn valid_problem_input() -> MemoryRecordInput<'static> {
        MemoryRecordInput {
            id: "mem.problem.1",
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
            created_at_ticks: 1,
        }
    }

    #[test]
    fn valid_decision_constructs_ok() {
        let record = MemoryRecord::new(valid_decision_input()).expect("valid decision constructs");
        assert_eq!(record.kind, MemoryKind::Decision);
    }

    #[test]
    fn decision_requires_entity_and_source() {
        let mut missing_entity = valid_decision_input();
        missing_entity.entity = "";
        assert_eq!(
            MemoryRecord::new(missing_entity),
            Err(MemoryRecordError::DecisionMissingEntity)
        );
        assert_eq!(
            MemoryRecordError::DecisionMissingEntity.reason(),
            "decision_missing_entity"
        );

        let mut missing_source = valid_decision_input();
        missing_source.source = MemorySource::new("", "");
        assert_eq!(
            MemoryRecord::new(missing_source),
            Err(MemoryRecordError::DecisionMissingSource)
        );
        assert_eq!(
            MemoryRecordError::DecisionMissingSource.reason(),
            "decision_missing_source"
        );
    }

    #[test]
    fn valid_problem_constructs_ok() {
        let record = MemoryRecord::new(valid_problem_input()).expect("valid problem constructs");
        assert_eq!(record.kind, MemoryKind::Problem);
    }

    #[test]
    fn problem_requires_entity_and_status() {
        let mut missing_entity = valid_problem_input();
        missing_entity.entity = "";
        assert_eq!(
            MemoryRecord::new(missing_entity),
            Err(MemoryRecordError::ProblemMissingEntity)
        );
        assert_eq!(
            MemoryRecordError::ProblemMissingEntity.reason(),
            "problem_missing_entity"
        );

        let mut missing_status = valid_problem_input();
        missing_status.predicate = "";
        assert_eq!(
            MemoryRecord::new(missing_status),
            Err(MemoryRecordError::ProblemMissingStatus)
        );
        assert_eq!(
            MemoryRecordError::ProblemMissingStatus.reason(),
            "problem_missing_status"
        );
    }

    #[test]
    fn decision_may_legitimately_supersede_a_non_audit_id() {
        let mut input = valid_decision_input();
        input.id = "mem.decision.2";
        input.supersedes = vec!["mem.decision.1"];
        let record = MemoryRecord::new(input).expect("decision superseding a non-audit id is ok");
        assert_eq!(record.supersedes, vec!["mem.decision.1"]);
    }

    #[test]
    fn audit_kinds_may_never_supersede() {
        for kind in [
            "capability_grant",
            "capability_denial",
            "promotion_tx_ref",
            "rollback_tx_ref",
            "export_audit",
        ] {
            let result = MemoryRecord::new(MemoryRecordInput {
                id: "mem.audit.1",
                kind,
                entity: "svc.x",
                predicate: "p",
                value: Value::Null,
                classification: "local_only",
                authority: "core_ledger",
                boot_id: "boot:1",
                sequence: 1,
                source: MemorySource::new("system.snapshot", "snapshot:1"),
                evidence: vec![],
                tags: vec![],
                supersedes: vec!["mem.some.prior.id"],
                created_at_ticks: 1,
            });
            assert_eq!(result, Err(MemoryRecordError::AuditKindMayNotSupersede));
        }
        assert_eq!(
            MemoryRecordError::AuditKindMayNotSupersede.reason(),
            "audit_kind_may_not_supersede"
        );
    }

    #[test]
    fn supersedes_list_too_long_is_rejected() {
        let mut input = valid_decision_input();
        input.supersedes = vec![
            "id.1", "id.2", "id.3", "id.4", "id.5", "id.6", "id.7", "id.8", "id.9",
        ];
        assert_eq!(input.supersedes.len(), MAX_SUPERSEDES_PER_RECORD + 1);
        let result = MemoryRecord::new(input);
        assert_eq!(result, Err(MemoryRecordError::SupersedesListTooLong));
        assert_eq!(result.unwrap_err().reason(), "supersedes_list_too_long");
    }

    #[test]
    fn supersedes_list_at_max_is_accepted() {
        let mut input = valid_decision_input();
        input.supersedes = vec![
            "id.1", "id.2", "id.3", "id.4", "id.5", "id.6", "id.7", "id.8",
        ];
        assert_eq!(input.supersedes.len(), MAX_SUPERSEDES_PER_RECORD);
        assert!(MemoryRecord::new(input).is_ok());
    }

    #[test]
    fn self_supersede_is_rejected() {
        let mut input = valid_decision_input();
        input.id = "mem.decision.self";
        input.supersedes = vec!["mem.decision.self"];
        let result = MemoryRecord::new(input);
        assert_eq!(result, Err(MemoryRecordError::SelfSupersede));
        assert_eq!(result.unwrap_err().reason(), "supersede_self_reference");
    }

    #[test]
    fn all_new_error_reasons_are_pairwise_unique() {
        let reasons = [
            MemoryRecordError::SecretNeverDurable.reason(),
            MemoryRecordError::UnknownKind.reason(),
            MemoryRecordError::ObservationMissingEntity.reason(),
            MemoryRecordError::ObservationMissingSource.reason(),
            MemoryRecordError::DecisionMissingEntity.reason(),
            MemoryRecordError::DecisionMissingSource.reason(),
            MemoryRecordError::ProblemMissingEntity.reason(),
            MemoryRecordError::ProblemMissingStatus.reason(),
            MemoryRecordError::AuditKindMayNotSupersede.reason(),
            MemoryRecordError::SupersedesListTooLong.reason(),
            MemoryRecordError::SelfSupersede.reason(),
        ];
        let mut idx = 0usize;
        while idx < reasons.len() {
            let mut other = idx + 1;
            while other < reasons.len() {
                assert_ne!(reasons[idx], reasons[other]);
                other += 1;
            }
            idx += 1;
        }
    }

    // -----------------------------------------------------------------------
    // M9C-1a reparser tests.
    // -----------------------------------------------------------------------

    fn render_payload(record: &MemoryRecord<'_>) -> Vec<u8> {
        let mut out = Vec::new();
        write_json(&record.to_record_value(), &mut out, 0);
        out
    }

    /// Asserts a rendered record round-trips through `parse` with identical
    /// metadata (the `value` sub-tree is intentionally absent from the view).
    fn assert_roundtrip(record: &MemoryRecord<'_>) {
        let payload = render_payload(record);
        let view = parse(&payload).expect("valid payload must reparse");
        assert_eq!(view.id, record.id);
        assert_eq!(view.kind, record.kind);
        assert_eq!(view.entity, record.entity);
        assert_eq!(view.predicate, record.predicate);
        assert_eq!(view.classification, record.classification);
        assert_eq!(view.authority, record.authority);
        assert_eq!(view.boot_id, record.boot_id);
        assert_eq!(view.sequence, record.sequence);
        assert_eq!(view.source, record.source);
        assert_eq!(view.evidence, record.evidence);
        assert_eq!(view.tags, record.tags);
        assert_eq!(view.supersedes, record.supersedes);
        assert_eq!(view.created_at_ticks, record.created_at_ticks);
    }

    #[test]
    fn reparse_round_trips_fixed_sample_record() {
        assert_roundtrip(&fixed_sample_record());
    }

    /// A valid record for the given kind, satisfying its per-kind invariants.
    fn valid_record_for_kind(kind: &'static str) -> MemoryRecord<'static> {
        MemoryRecord::new(MemoryRecordInput {
            id: "mem.roundtrip.1",
            kind,
            entity: "svc.provider.openai_direct",
            predicate: "provider_trust_denied",
            value: Value::Str("payload-value-is-skipped"),
            classification: "local_only",
            authority: "core_ledger",
            boot_id: "boot:0000000000000001",
            sequence: 7,
            source: MemorySource::new("system.snapshot", "snapshot:current_boot.00000003"),
            evidence: vec!["problem:provider.tls_pin_config_missing"],
            tags: vec!["provider", "trust"],
            supersedes: vec![],
            created_at_ticks: 999,
        })
        .expect("kind fixture must construct")
    }

    #[test]
    fn reparse_round_trips_all_eight_kinds() {
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
            assert_roundtrip(&valid_record_for_kind(kind));
        }
    }

    #[test]
    fn reparse_round_trips_a_superseding_record() {
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
        .expect("superseding record constructs");
        let payload = render_payload(&superseding);
        let view = parse(&payload).expect("reparses");
        assert_eq!(view.supersedes, vec!["mem.problem.open.1"]);
    }

    fn quote(value: &str) -> String {
        let mut out = String::from("\"");
        out.push_str(value);
        out.push('"');
        out
    }

    /// The canonical field list of a minimal-valid `decision` payload, each entry
    /// `(key, raw-json-value)`. The reparser is whitespace-tolerant, so this
    /// compact (space-free) canonical form parses identically to `write_json`.
    fn canonical_decision_fields() -> Vec<(&'static str, String)> {
        vec![
            ("schema", quote(SCHEMA)),
            ("id", quote("mem.decision.hand")),
            ("kind", quote("decision")),
            ("entity", quote("adr.0004")),
            ("predicate", quote("memory_is_typed_state")),
            ("value", "null".to_string()),
            ("classification", quote("local_only")),
            ("authority", quote("decision")),
            ("boot_id", quote("boot:1")),
            ("sequence", "1".to_string()),
            (
                "source",
                "{\"method\":\"system.snapshot\",\"record_id\":\"snapshot:1\"}".to_string(),
            ),
            ("evidence", "[]".to_string()),
            ("tags", "[]".to_string()),
            ("supersedes", "[]".to_string()),
            (
                "created_at",
                "{\"clock\":\"boot_relative\",\"ticks\":1}".to_string(),
            ),
        ]
    }

    fn build_payload(fields: &[(&str, String)]) -> Vec<u8> {
        let mut out = String::from("{");
        for (idx, (key, raw)) in fields.iter().enumerate() {
            if idx != 0 {
                out.push(',');
            }
            out.push_str(&quote(key));
            out.push(':');
            out.push_str(raw);
        }
        out.push('}');
        out.into_bytes()
    }

    #[test]
    fn reparse_hand_built_canonical_decision_is_ok() {
        let payload = build_payload(&canonical_decision_fields());
        let view = parse(&payload).expect("canonical parses");
        assert_eq!(view.kind, MemoryKind::Decision);
        assert_eq!(view.id, "mem.decision.hand");
        assert_eq!(view.classification, Classification::LocalOnly);
        assert_eq!(view.created_at_ticks, 1);
    }

    /// Every truncation of a valid payload must be rejected — never `Ok`, never a
    /// panic. This blankets the mid-value / mid-array / mid-final-brace cases.
    #[test]
    fn reparse_every_truncation_is_fail_closed() {
        let payload = render_payload(&fixed_sample_record());
        assert!(parse(&payload).is_ok());
        for cut in 0..payload.len() {
            assert!(
                parse(&payload[..cut]).is_err(),
                "truncation to {cut} bytes must be rejected"
            );
        }
    }

    #[test]
    fn reparse_specific_truncations_hit_expected_errors() {
        // Mid-value: cut inside the (skipped) value string -> ValueUnbalanced.
        let mut fields = canonical_decision_fields();
        fields[5].1 = quote("SENTINEL_VALUE");
        let full = build_payload(&fields);
        let marker = b"SENTINEL";
        let at = full
            .windows(marker.len())
            .position(|window| window == marker)
            .expect("marker present");
        assert_eq!(
            parse(&full[..at + 3]),
            Err(MemoryRecordParseError::ValueUnbalanced)
        );

        // Mid-array: cut inside an evidence string element -> BadString.
        let mut fields = canonical_decision_fields();
        fields[11].1 = "[\"problem:PINPOINT\"]".to_string();
        let full = build_payload(&fields);
        let marker = b"PINPOINT";
        let at = full
            .windows(marker.len())
            .position(|window| window == marker)
            .expect("marker present");
        assert_eq!(
            parse(&full[..at + 3]),
            Err(MemoryRecordParseError::BadString)
        );

        // Mid-final-brace: drop the last byte (the closing brace) -> MissingField.
        let full = render_payload(&fixed_sample_record());
        assert_eq!(
            parse(&full[..full.len() - 1]),
            Err(MemoryRecordParseError::MissingField)
        );
    }

    #[test]
    fn reparse_non_memory_schema_is_dropped() {
        let mut fields = canonical_decision_fields();
        fields[0].1 = quote("raios.promotion_transaction.v0");
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::SchemaMismatch)
        );
    }

    #[test]
    fn reparse_trailing_garbage_is_rejected() {
        let mut payload = render_payload(&fixed_sample_record());
        payload.extend_from_slice(b"garbage");
        assert_eq!(parse(&payload), Err(MemoryRecordParseError::TrailingBytes));
        // Trailing whitespace only is fine.
        let mut ok = render_payload(&fixed_sample_record());
        ok.extend_from_slice(b"\r\n  ");
        assert!(parse(&ok).is_ok());
    }

    #[test]
    fn reparse_missing_field_is_rejected() {
        // Drop `predicate` entirely; the next key seen is the later `value`.
        let mut fields = canonical_decision_fields();
        fields.remove(4);
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::MissingField)
        );
    }

    #[test]
    fn reparse_reordered_field_is_rejected() {
        // Rename `id`'s slot to a duplicate earlier key (`schema`) -> out of order.
        let mut fields = canonical_decision_fields();
        fields[1].0 = "schema";
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::FieldOrder)
        );
    }

    #[test]
    fn reparse_extra_unknown_key_is_rejected() {
        // Put an unknown key where `id` is expected.
        let mut fields = canonical_decision_fields();
        fields[1].0 = "bonus";
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::ExtraField)
        );
    }

    #[test]
    fn reparse_value_unterminated_string_is_value_unbalanced() {
        let mut fields = canonical_decision_fields();
        fields[5].1 = quote("SENTINEL_VALUE");
        let full = build_payload(&fields);
        let marker = b"SENTINEL";
        let at = full
            .windows(marker.len())
            .position(|window| window == marker)
            .expect("marker present");
        assert_eq!(
            parse(&full[..at + 3]),
            Err(MemoryRecordParseError::ValueUnbalanced)
        );
    }

    #[test]
    fn reparse_value_unbalanced_brace_is_value_unbalanced() {
        // Two unmatched opens; even after balancing the rest of the record and
        // the final top-level brace, one open remains -> unbalanced at EOF.
        let mut fields = canonical_decision_fields();
        fields[5].1 = "{{".to_string();
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::ValueUnbalanced)
        );
    }

    #[test]
    fn reparse_value_over_depth_is_value_unbalanced() {
        let mut fields = canonical_decision_fields();
        let mut deep = String::new();
        for _ in 0..40 {
            deep.push('[');
        }
        for _ in 0..40 {
            deep.push(']');
        }
        fields[5].1 = deep;
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::ValueUnbalanced)
        );
    }

    #[test]
    fn reparse_secret_classification_is_rejected() {
        let mut fields = canonical_decision_fields();
        fields[6].1 = quote("secret");
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::SecretClassification)
        );
    }

    #[test]
    fn reparse_unknown_kind_is_rejected() {
        let mut fields = canonical_decision_fields();
        fields[2].1 = quote("chat_history");
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::UnknownKind)
        );
    }

    #[test]
    fn reparse_non_ascii_byte_is_rejected() {
        let mut payload = render_payload(&fixed_sample_record());
        let mid = payload.len() / 2;
        payload[mid] = 0xff;
        assert_eq!(parse(&payload), Err(MemoryRecordParseError::NotAscii));
    }

    #[test]
    fn reparse_bad_number_is_rejected() {
        // Non-canonical leading zero on `sequence`.
        let mut fields = canonical_decision_fields();
        fields[9].1 = "01".to_string();
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::BadNumber)
        );
    }

    #[test]
    fn reparse_observation_requires_entity_and_source() {
        let mut fields = canonical_decision_fields();
        fields[2].1 = quote("observation");
        fields[3].1 = quote(""); // empty entity
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::ObservationMissingEntity)
        );

        let mut fields = canonical_decision_fields();
        fields[2].1 = quote("observation");
        fields[10].1 = "{\"method\":\"\",\"record_id\":\"\"}".to_string();
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::ObservationMissingSource)
        );
    }

    #[test]
    fn reparse_decision_requires_entity_and_source() {
        let mut fields = canonical_decision_fields();
        fields[3].1 = quote("");
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::DecisionMissingEntity)
        );

        let mut fields = canonical_decision_fields();
        fields[10].1 = "{\"method\":\"m\",\"record_id\":\"\"}".to_string();
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::DecisionMissingSource)
        );
    }

    #[test]
    fn reparse_problem_requires_entity_and_status() {
        let mut fields = canonical_decision_fields();
        fields[2].1 = quote("problem");
        fields[3].1 = quote("");
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::ProblemMissingEntity)
        );

        let mut fields = canonical_decision_fields();
        fields[2].1 = quote("problem");
        fields[4].1 = quote(""); // empty predicate/status
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::ProblemMissingStatus)
        );
    }

    #[test]
    fn reparse_supersedes_too_long_is_rejected() {
        let mut fields = canonical_decision_fields();
        fields[13].1 = "[\"a\",\"b\",\"c\",\"d\",\"e\",\"f\",\"g\",\"h\",\"i\"]".to_string();
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::SupersedesTooLong)
        );
    }

    #[test]
    fn reparse_self_supersede_is_rejected() {
        let mut fields = canonical_decision_fields();
        fields[1].1 = quote("mem.self");
        fields[13].1 = "[\"mem.self\"]".to_string();
        assert_eq!(
            parse(&build_payload(&fields)),
            Err(MemoryRecordParseError::SelfSupersede)
        );
    }

    #[test]
    fn reparse_error_reasons_are_pairwise_unique() {
        let reasons = [
            MemoryRecordParseError::NotAscii.reason(),
            MemoryRecordParseError::SchemaMismatch.reason(),
            MemoryRecordParseError::UnknownKind.reason(),
            MemoryRecordParseError::SecretClassification.reason(),
            MemoryRecordParseError::MissingField.reason(),
            MemoryRecordParseError::FieldOrder.reason(),
            MemoryRecordParseError::ExtraField.reason(),
            MemoryRecordParseError::TrailingBytes.reason(),
            MemoryRecordParseError::ValueUnbalanced.reason(),
            MemoryRecordParseError::BadNumber.reason(),
            MemoryRecordParseError::BadString.reason(),
            MemoryRecordParseError::ObservationMissingEntity.reason(),
            MemoryRecordParseError::ObservationMissingSource.reason(),
            MemoryRecordParseError::DecisionMissingEntity.reason(),
            MemoryRecordParseError::DecisionMissingSource.reason(),
            MemoryRecordParseError::ProblemMissingEntity.reason(),
            MemoryRecordParseError::ProblemMissingStatus.reason(),
            MemoryRecordParseError::SupersedesTooLong.reason(),
            MemoryRecordParseError::SelfSupersede.reason(),
        ];
        let mut idx = 0usize;
        while idx < reasons.len() {
            let mut other = idx + 1;
            while other < reasons.len() {
                assert_ne!(reasons[idx], reasons[other]);
                other += 1;
            }
            idx += 1;
        }
    }

    #[test]
    fn reparse_view_never_exposes_value() {
        // The view type has no `value` field; confirm a record whose value is a
        // large escaped-text tree still reparses (value skipped, not decoded).
        let record = MemoryRecord::new(MemoryRecordInput {
            id: "mem.obs.value",
            kind: "observation",
            entity: "svc.x",
            predicate: "p",
            value: Value::Str("line1\nline2\t\"quoted\"\\backslash"),
            classification: "local_only",
            authority: "event",
            boot_id: "boot:1",
            sequence: 1,
            source: MemorySource::new("system.snapshot", "snapshot:1"),
            evidence: vec![],
            tags: vec![],
            supersedes: vec![],
            created_at_ticks: 1,
        })
        .expect("record constructs");
        let payload = render_payload(&record);
        let view: MemoryRecordView<'_> = parse(&payload).expect("reparses");
        assert_eq!(view.entity, "svc.x");
    }
}
