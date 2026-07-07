//! Pinned recovery-lifeline command table (M8A-1, evidence-only).
//!
//! The recovery lifeline is a minimal, serial-first path that must survive when
//! the world above it breaks. This module is the SINGLE source of truth for the
//! frozen set of lifeline method names, their honest `implemented` flags, and a
//! `vocabulary_sha256` fingerprint over the logical table. The kernel checks this
//! table on a dedicated dispatch path (BEFORE the general `AGENT_METHODS` table)
//! and renders it as `raios.recovery_lifeline_table.v0`; the fingerprint is a
//! pin — silent authority growth (a new endpoint, an `implemented` flip) changes
//! the hash and fails the VM/host gate.
//!
//! M8A-1 grants nothing: only `recovery.lifeline_table` is implemented (a pure
//! table read); every other endpoint is registered but denied. The lifeline never
//! promotes new code and is honestly `dev_key_not_owner_sealed`, never owner_sealed.

use alloc::{vec, vec::Vec};

use crate::record::{sha256_of_json, Field, Value};

pub const SCHEMA: &str = "raios.recovery_lifeline_table.v0";
pub const SNAPSHOT_SCHEMA: &str = "raios.recovery_snapshot.v0";
pub const SCOPE: &str = "current_boot";
pub const CLASSIFICATION: &str = "local_only";
pub const TRANSPORT: &str = "serial_local";
pub const TRUST_STATE: &str = "local_physical_console";
pub const TRUST_TIER: &str = "dev_key_not_owner_sealed";

pub const METHOD_LIFELINE_TABLE: &str = "recovery.lifeline_table";
pub const METHOD_SNAPSHOT: &str = "recovery.snapshot";
pub const METHOD_RESTART_LAST_GOOD: &str = "recovery.restart_last_good";
pub const METHOD_DISABLE_MODULE: &str = "recovery.disable_module";
pub const METHOD_ROLLBACK: &str = "recovery.rollback";
pub const METHOD_LOAD_ARTIFACT_BY_HASH: &str = "recovery.load_artifact_by_hash";

/// One frozen lifeline endpoint. `implemented=false` endpoints are registered so
/// the dispatch boundary and denial shape exist, but they mutate nothing yet.
#[derive(Clone, Copy)]
pub struct LifelineMethod {
    pub name: &'static str,
    pub capability: &'static str,
    pub implemented: bool,
    pub mutating: bool,
}

/// The frozen table. Order is pinned — it feeds the vocabulary fingerprint.
pub const LIFELINE_METHODS: &[LifelineMethod] = &[
    LifelineMethod {
        name: METHOD_LIFELINE_TABLE,
        capability: "cap.recovery.lifeline_table.read",
        implemented: true,
        mutating: false,
    },
    LifelineMethod {
        name: METHOD_SNAPSHOT,
        capability: "cap.recovery.snapshot.read",
        implemented: true,
        mutating: false,
    },
    LifelineMethod {
        name: METHOD_RESTART_LAST_GOOD,
        capability: "cap.recovery.restart_last_good",
        implemented: false,
        mutating: true,
    },
    LifelineMethod {
        name: METHOD_DISABLE_MODULE,
        capability: "cap.recovery.disable_module",
        implemented: true,
        mutating: true,
    },
    LifelineMethod {
        name: METHOD_ROLLBACK,
        capability: "cap.recovery.rollback",
        implemented: false,
        mutating: true,
    },
    LifelineMethod {
        name: METHOD_LOAD_ARTIFACT_BY_HASH,
        capability: "cap.recovery.load_artifact_by_hash",
        implemented: false,
        mutating: true,
    },
];

/// Match a method name against the frozen table. Full-string, ASCII
/// case-insensitive (via `method_eq`, consistent with the rest of the agent
/// protocol) so the restore-only path stays usable when things are broken; no
/// head/prefix/alias matching. NOTE (M8A-2): endpoints that will take arguments
/// (`recovery.snapshot`/`rollback`/`load_artifact_by_hash`) must move to
/// head-token matching once implemented, or they become unreachable with args.
pub fn lookup(method: &str) -> Option<&'static LifelineMethod> {
    let mut idx = 0usize;
    while idx < LIFELINE_METHODS.len() {
        if crate::method_eq(LIFELINE_METHODS[idx].name, method) {
            return Some(&LIFELINE_METHODS[idx]);
        }
        idx += 1;
    }
    None
}

fn method_rows() -> Vec<Value<'static>> {
    let mut rows = Vec::new();
    let mut idx = 0usize;
    while idx < LIFELINE_METHODS.len() {
        let m = &LIFELINE_METHODS[idx];
        rows.push(Value::InlineObject(vec![
            Field::new("name", Value::Str(m.name)),
            Field::new("capability", Value::Str(m.capability)),
            Field::new("implemented", Value::Bool(m.implemented)),
            Field::new("mutating", Value::Bool(m.mutating)),
        ]));
        idx += 1;
    }
    rows
}

/// The logical table fields (WITHOUT the self-referential fingerprint). This is
/// exactly what `vocabulary_sha256` hashes and what the kernel renders (then
/// appends the fingerprint to).
pub fn build_core_fields() -> Vec<Field<'static>> {
    vec![
        Field::new("schema", Value::Str(SCHEMA)),
        Field::new("scope", Value::Str(SCOPE)),
        Field::new("classification", Value::Str(CLASSIFICATION)),
        Field::new("transport", Value::Str(TRANSPORT)),
        Field::new("trust_state", Value::Str(TRUST_STATE)),
        Field::new("trust_tier", Value::Str(TRUST_TIER)),
        Field::new("owner_sealed", Value::Bool(false)),
        Field::new("routes_through_wasm", Value::Bool(false)),
        Field::new("routes_through_provider", Value::Bool(false)),
        Field::new("mutates_state", Value::Bool(false)),
        Field::new("method_count", Value::U64(LIFELINE_METHODS.len() as u64)),
        Field::new("methods", Value::Array(method_rows())),
    ]
}

/// Canonical fingerprint over the logical table. A stable pin: any endpoint added,
/// removed, reordered, or flipped to `implemented` changes this value.
pub fn vocabulary_sha256() -> [u8; 32] {
    sha256_of_json(&Value::Object(build_core_fields()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sha256_hex;

    #[test]
    fn table_names_and_flags_are_frozen() {
        let names: Vec<&str> = LIFELINE_METHODS.iter().map(|m| m.name).collect();
        assert_eq!(
            names,
            vec![
                "recovery.lifeline_table",
                "recovery.snapshot",
                "recovery.restart_last_good",
                "recovery.disable_module",
                "recovery.rollback",
                "recovery.load_artifact_by_hash",
            ]
        );
        // Implemented: M8A-1 table, M8A-2 snapshot, M8B-1 disable_module executor;
        // the other three mutators stay denied.
        assert!(lookup("recovery.lifeline_table").unwrap().implemented);
        assert!(lookup("recovery.snapshot").unwrap().implemented);
        assert!(lookup("recovery.disable_module").unwrap().implemented);
        for name in [
            "recovery.restart_last_good",
            "recovery.rollback",
            "recovery.load_artifact_by_hash",
        ] {
            assert!(!lookup(name).unwrap().implemented, "{name} must be denied");
        }
        // The four state-changing endpoints are marked mutating; reads are not.
        assert!(!lookup("recovery.lifeline_table").unwrap().mutating);
        assert!(!lookup("recovery.snapshot").unwrap().mutating);
        for name in [
            "recovery.restart_last_good",
            "recovery.disable_module",
            "recovery.rollback",
            "recovery.load_artifact_by_hash",
        ] {
            assert!(lookup(name).unwrap().mutating, "{name} must be mutating");
        }
        assert!(lookup("recovery.not_a_method").is_none());
    }

    #[test]
    fn vocabulary_hash_is_pinned() {
        // Golden fingerprint of the M8A-1 logical table. If this changes, the
        // lifeline vocabulary changed — re-derive intentionally, never blindly.
        let hex = sha256_hex(&vocabulary_sha256());
        let hex = core::str::from_utf8(&hex).unwrap();
        assert_eq!(
            hex,
            "03d3985c104de4d025c7b38aa6d484bbe68e0eb3512bc034993663c7b7a112a3"
        );
    }
}
