//! `raios.recovery_load.v0` response vocabulary for `recovery.load_artifact_by_hash`
//! (M8D-1: hash-selected FULL re-verify that GRANTS NOTHING).
//!
//! This module is the single source of truth for the recovery-load response schema
//! names, the fail-closed denial reasons, and the always-false / honest-posture label
//! set every recovery-load response and selftest case carries. M8D-1 re-verifies the
//! full M6 evidence chain for a locally-stored artifact and REPORTS the decision; it
//! never retains, loads, starts, fetches, accepts external bytes/URLs, grants a new
//! capability, or writes anything durable. Centralising the honest labels here (and
//! host-testing them) keeps the wire shape from silently claiming authority it lacks.

use alloc::{vec, vec::Vec};

use crate::record::{Field, Value};

pub const SCHEMA: &str = "raios.recovery_load.v0";
pub const SELFTEST_SCHEMA: &str = "raios.recovery_load_selftest.v0";
pub const RESPONSE_ID: &str = "recovery_load.current_boot.load_artifact_by_hash.v0";
pub const METHOD: &str = "recovery.load_artifact_by_hash";
pub const SELFTEST_METHOD: &str = "recovery.load_artifact_by_hash_selftest";
pub const SCOPE: &str = "current_boot";
pub const CLASSIFICATION: &str = "local_only";
pub const TRUST_TIER: &str = "dev_key_not_owner_sealed";

/// Fail-closed denial reasons. Parse failure reads NOTHING; SAFE/PersistenceUnavailable
/// is rejected BEFORE any store read; an absent record denies without a reverify.
pub const REASON_MALFORMED_HASH: &str = "malformed_hash";
pub const REASON_SAFE_MODE: &str = "boot_control_safe_mode";
pub const REASON_CONTROLLER_ABSENT: &str = "ahci_controller_not_observed";
pub const REASON_NOT_IN_LOCAL_STORE: &str = "artifact_not_in_local_store";

pub const STATUS_DENIED: &str = "capability_denied";
/// The full chain re-verified, but M8D-1 STILL loads nothing (that authority is M8D-2).
pub const STATUS_REVERIFIED_LOAD_DENIED: &str = "reverified_load_denied_grants_nothing";
pub const REASON_REVERIFIED_LOAD_DENIED: &str = "reverified_but_load_grants_nothing_m8d1";
pub const REVERIFY_NOT_ATTEMPTED: &str = "reverify_not_attempted";

/// The always-false / honest-posture label set shared by EVERY recovery-load response
/// and selftest case. M8D-1 grants nothing: it never loads, starts, retains, fetches,
/// accepts external bytes or a URL, mutates live state, or writes anything durable. These
/// labels are constant so the wire shape cannot silently claim authority it does not have.
pub fn grants_nothing_fields() -> Vec<Field<'static>> {
    vec![
        Field::new("trust_tier", Value::Str(TRUST_TIER)),
        Field::new("owner_sealed", Value::Bool(false)),
        Field::new("promotion_authority_is_placeholder", Value::Bool(true)),
        Field::new("grants_new_capability", Value::Bool(false)),
        Field::new("authorizes_load", Value::Bool(false)),
        Field::new("cross_reboot_proven", Value::Bool(false)),
        Field::new("service_loaded", Value::Bool(false)),
        Field::new("load_still_denied", Value::Bool(true)),
        Field::new("persistence_claimed", Value::Bool(false)),
        Field::new("mutates_live_state", Value::Bool(false)),
        Field::new("accepts_external_bytes", Value::Bool(false)),
        Field::new("accepts_url", Value::Bool(false)),
        Field::new("fetches", Value::Bool(false)),
        Field::new("routes_through_wasm", Value::Bool(false)),
        Field::new("routes_through_provider", Value::Bool(false)),
        Field::new("durable_write_attempted", Value::Bool(false)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::Value;

    #[test]
    fn honest_labels_grant_nothing() {
        for field in grants_nothing_fields() {
            match (field.key, field.value) {
                // The only non-false labels are the two honest facts: the placeholder
                // promotion authority, and that the load is (still) denied this slice.
                ("promotion_authority_is_placeholder", Value::Bool(value)) => assert!(value),
                ("load_still_denied", Value::Bool(value)) => assert!(value),
                ("trust_tier", Value::Str(value)) => assert_eq!(value, TRUST_TIER),
                (key, Value::Bool(value)) => {
                    assert!(!value, "grants-nothing label {key} must be false")
                }
                (key, _) => panic!("unexpected grants-nothing field shape: {key}"),
            }
        }
    }
}
