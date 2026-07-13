//! Typed field-table construction and fail-closed decision records.

use alloc::{vec, vec::Vec};

use crate::record::{Field, Value};

pub enum Cell<'a> {
    Null,
    Bool(bool),
    U64(u64),
    Str(&'a str),
    Sha256([u8; 32]),
    EventSequence(u64),
    Value(Value<'a>),
}

pub struct FieldSpec<T> {
    pub key: &'static str,
    pub get: for<'a> fn(&'a T) -> Cell<'a>,
}

pub fn object_from<'a, T>(table: &[FieldSpec<T>], source: &'a T) -> Value<'a> {
    let mut fields = Vec::with_capacity(table.len());
    for spec in table {
        debug_assert!(
            !is_reserved_decision_key(spec.key),
            "decision key is reserved: {}",
            spec.key
        );
        fields.push(Field::new(spec.key, cell_value((spec.get)(source))));
    }
    Value::InlineObject(fields)
}

fn cell_value(cell: Cell<'_>) -> Value<'_> {
    match cell {
        Cell::Null => Value::Null,
        Cell::Bool(value) => Value::Bool(value),
        Cell::U64(value) => Value::U64(value),
        Cell::Str(value) => Value::Str(value),
        Cell::Sha256(value) => Value::Sha256(value),
        Cell::EventSequence(value) => Value::EventSequence(value),
        Cell::Value(value) => value,
    }
}

fn is_reserved_decision_key(key: &str) -> bool {
    matches!(key, "outcome" | "grants" | "effects" | "blocked_by") || key.starts_with("authorizes_")
}

#[macro_export]
macro_rules! field {
    ($key:literal, null <- $accessor:path) => {
        $crate::record_table::FieldSpec {
            key: $key,
            get: |source| {
                $accessor(source);
                $crate::record_table::Cell::Null
            },
        }
    };
    ($key:literal, bool <- $accessor:path) => {
        $crate::record_table::FieldSpec {
            key: $key,
            get: |source| $crate::record_table::Cell::Bool($accessor(source)),
        }
    };
    ($key:literal, u64 <- $accessor:path) => {
        $crate::record_table::FieldSpec {
            key: $key,
            get: |source| $crate::record_table::Cell::U64($accessor(source)),
        }
    };
    ($key:literal, str <- $accessor:path) => {
        $crate::record_table::FieldSpec {
            key: $key,
            get: |source| $crate::record_table::Cell::Str($accessor(source)),
        }
    };
    ($key:literal, sha256 <- $accessor:path) => {
        $crate::record_table::FieldSpec {
            key: $key,
            get: |source| $crate::record_table::Cell::Sha256($accessor(source)),
        }
    };
    ($key:literal, event_sequence <- $accessor:path) => {
        $crate::record_table::FieldSpec {
            key: $key,
            get: |source| $crate::record_table::Cell::EventSequence($accessor(source)),
        }
    };
    ($key:literal, value <- $accessor:path) => {
        $crate::record_table::FieldSpec {
            key: $key,
            get: |source| $crate::record_table::Cell::Value($accessor(source)),
        }
    };
}

pub struct Observation<'a> {
    reason: &'a str,
}

impl<'a> Observation<'a> {
    #[allow(dead_code)] // P4-0 substrate; evaluator consumers land in later slices.
    pub(crate) const fn new(reason: &'a str) -> Self {
        Self { reason }
    }
}

pub struct DeniedBy<'a> {
    evidence_id: &'a str,
    status: &'a str,
    reason: &'a str,
}

impl<'a> DeniedBy<'a> {
    #[allow(dead_code)] // P4-0 substrate; evaluator consumers land in later slices.
    pub(crate) const fn new(evidence_id: &'a str, status: &'a str, reason: &'a str) -> Self {
        Self {
            evidence_id,
            status,
            reason,
        }
    }
}

pub struct DenialDecision<'a> {
    reason: &'a str,
    requested_capability: &'a str,
    blocked_by: &'a [DeniedBy<'a>],
}

impl<'a> DenialDecision<'a> {
    #[allow(dead_code)] // P4-0 substrate; evaluator consumers land in later slices.
    pub(crate) const fn new(
        reason: &'a str,
        requested_capability: &'a str,
        blocked_by: &'a [DeniedBy<'a>],
    ) -> Self {
        Self {
            reason,
            requested_capability,
            blocked_by,
        }
    }
}

/// Zero-sized evidence that only a raios-core evaluator can create.
pub struct GrantProof {
    _private: (),
}

impl GrantProof {
    #[allow(dead_code)] // P4-0 substrate; evaluator consumers land in later slices.
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }
}

pub struct GrantDecision<'a> {
    reason: &'a str,
    requested_capability: &'a str,
    grants: &'a [&'a str],
    effects: &'a [&'a str],
    _proof: GrantProof,
}

impl<'a> GrantDecision<'a> {
    #[allow(dead_code)] // P4-0 substrate; evaluator consumers land in later slices.
    pub(crate) const fn new(
        proof: GrantProof,
        reason: &'a str,
        requested_capability: &'a str,
        grants: &'a [&'a str],
        effects: &'a [&'a str],
    ) -> Self {
        Self {
            reason,
            requested_capability,
            grants,
            effects,
            _proof: proof,
        }
    }
}

pub enum EvaluatedDecision<'a> {
    Observed(&'a Observation<'a>),
    Denied(&'a DenialDecision<'a>),
    Granted(&'a GrantDecision<'a>),
}

pub fn decision_value<'a>(decision: &EvaluatedDecision<'a>) -> Value<'a> {
    match decision {
        EvaluatedDecision::Observed(observation) => Value::InlineObject(vec![
            Field::new("outcome", Value::Str("observed")),
            Field::new("reason", Value::Str(observation.reason)),
        ]),
        EvaluatedDecision::Denied(denial) => Value::InlineObject(vec![
            Field::new("outcome", Value::Str("denied")),
            Field::new("reason", Value::Str(denial.reason)),
            Field::new(
                "requested_capability",
                Value::Str(denial.requested_capability),
            ),
            Field::new("grants", Value::InlineArray(vec![])),
            Field::new("effects", Value::InlineArray(vec![])),
            Field::new(
                "blocked_by",
                Value::InlineArray(
                    denial
                        .blocked_by
                        .iter()
                        .map(|blocked| {
                            Value::InlineObject(vec![
                                Field::new("evidence_id", Value::Str(blocked.evidence_id)),
                                Field::new("status", Value::Str(blocked.status)),
                                Field::new("reason", Value::Str(blocked.reason)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]),
        EvaluatedDecision::Granted(grant) => Value::InlineObject(vec![
            Field::new("outcome", Value::Str("granted")),
            Field::new("reason", Value::Str(grant.reason)),
            Field::new(
                "requested_capability",
                Value::Str(grant.requested_capability),
            ),
            Field::new(
                "grants",
                Value::InlineArray(grant.grants.iter().copied().map(Value::Str).collect()),
            ),
            Field::new(
                "effects",
                Value::InlineArray(grant.effects.iter().copied().map(Value::Str).collect()),
            ),
        ]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{sha256_of_json, write_json};

    const DIGEST: [u8; 32] = [0x5a; 32];

    struct Fixture<'a> {
        text: &'a str,
        nested: &'a str,
    }

    impl Fixture<'_> {
        fn nothing(&self) {}
        fn flag(&self) -> bool {
            true
        }
        fn count(&self) -> u64 {
            42
        }
        fn text(&self) -> &str {
            self.text
        }
        fn digest(&self) -> [u8; 32] {
            DIGEST
        }
        fn sequence(&self) -> u64 {
            7
        }
        fn nested(&self) -> Value<'_> {
            Value::InlineObject(vec![Field::new("name", Value::Str(self.nested))])
        }
    }

    const FIELDS: &[FieldSpec<Fixture<'_>>] = &[
        field!("nothing", null <- Fixture::nothing),
        field!("flag", bool <- Fixture::flag),
        field!("count", u64 <- Fixture::count),
        field!("text", str <- Fixture::text),
        field!("digest", sha256 <- Fixture::digest),
        field!("event_id", event_sequence <- Fixture::sequence),
        field!("nested", value <- Fixture::nested),
    ];

    #[test]
    fn table_rendering_matches_hand_built_value_hash() {
        let fixture = Fixture {
            text: "hello",
            nested: "child",
        };
        let hand_built = Value::InlineObject(vec![
            Field::new("nothing", Value::Null),
            Field::new("flag", Value::Bool(true)),
            Field::new("count", Value::U64(42)),
            Field::new("text", Value::Str("hello")),
            Field::new("digest", Value::Sha256(DIGEST)),
            Field::new("event_id", Value::EventSequence(7)),
            Field::new(
                "nested",
                Value::InlineObject(vec![Field::new("name", Value::Str("child"))]),
            ),
        ]);

        assert_eq!(
            sha256_of_json(&object_from(FIELDS, &fixture)),
            sha256_of_json(&hand_built)
        );
    }

    #[test]
    fn cell_variants_map_to_existing_value_variants() {
        let fixture = Fixture {
            text: "hello",
            nested: "child",
        };
        let value = object_from(FIELDS, &fixture);
        assert_eq!(
            value,
            Value::InlineObject(vec![
                Field::new("nothing", Value::Null),
                Field::new("flag", Value::Bool(true)),
                Field::new("count", Value::U64(42)),
                Field::new("text", Value::Str("hello")),
                Field::new("digest", Value::Sha256(DIGEST)),
                Field::new("event_id", Value::EventSequence(7)),
                Field::new(
                    "nested",
                    Value::InlineObject(vec![Field::new("name", Value::Str("child"))]),
                ),
            ])
        );
        let mut rendered = std::vec::Vec::new();
        write_json(&value, &mut rendered, 0);
        assert_eq!(
            rendered,
            b"{\"nothing\": null, \"flag\": true, \"count\": 42, \"text\": \"hello\", \"digest\": \"sha256:5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a\", \"event_id\": \"event.current_boot.00000007\", \"nested\": {\"name\": \"child\"}}"
        );
    }

    #[test]
    fn object_from_rejects_reserved_decision_keys() {
        let fixture = Fixture {
            text: "hello",
            nested: "child",
        };
        for key in [
            "outcome",
            "grants",
            "effects",
            "blocked_by",
            "authorizes_load",
        ] {
            let table = [FieldSpec {
                key,
                get: |source: &Fixture<'_>| Cell::Bool(source.flag()),
            }];
            assert!(std::panic::catch_unwind(|| object_from(&table, &fixture)).is_err());
        }
    }

    #[test]
    fn denial_always_has_empty_grants_and_effects_in_evaluator_order() {
        let blocked_by = [
            DeniedBy::new("first", "missing", "first_missing"),
            DeniedBy::new("second", "rejected", "second_rejected"),
        ];
        let denial = DenialDecision::new("first_missing", "cap.example", &blocked_by);

        assert_eq!(
            decision_value(&EvaluatedDecision::Denied(&denial)),
            Value::InlineObject(vec![
                Field::new("outcome", Value::Str("denied")),
                Field::new("reason", Value::Str("first_missing")),
                Field::new("requested_capability", Value::Str("cap.example")),
                Field::new("grants", Value::InlineArray(vec![])),
                Field::new("effects", Value::InlineArray(vec![])),
                Field::new(
                    "blocked_by",
                    Value::InlineArray(vec![
                        Value::InlineObject(vec![
                            Field::new("evidence_id", Value::Str("first")),
                            Field::new("status", Value::Str("missing")),
                            Field::new("reason", Value::Str("first_missing")),
                        ]),
                        Value::InlineObject(vec![
                            Field::new("evidence_id", Value::Str("second")),
                            Field::new("status", Value::Str("rejected")),
                            Field::new("reason", Value::Str("second_rejected")),
                        ]),
                    ]),
                ),
            ])
        );
    }

    #[test]
    fn grant_decision_requires_evaluator_proof() {
        const GRANTS: &[&str] = &["cap.example"];
        const EFFECTS: &[&str] = &["scoped_effect"];
        let constructor: fn(
            GrantProof,
            &'static str,
            &'static str,
            &'static [&'static str],
            &'static [&'static str],
        ) -> GrantDecision<'static> = GrantDecision::new;
        let proof = GrantProof::new();
        let grant = constructor(
            proof,
            "scoped_policy_granted",
            "cap.example",
            GRANTS,
            EFFECTS,
        );

        assert_eq!(core::mem::size_of::<GrantProof>(), 0);
        assert!(matches!(
            decision_value(&EvaluatedDecision::Granted(&grant)),
            Value::InlineObject(_)
        ));
    }

    #[test]
    fn observation_carries_no_grants() {
        let observation = Observation::new("snapshot_returned");
        assert_eq!(
            decision_value(&EvaluatedDecision::Observed(&observation)),
            Value::InlineObject(vec![
                Field::new("outcome", Value::Str("observed")),
                Field::new("reason", Value::Str("snapshot_returned")),
            ])
        );
    }
}
