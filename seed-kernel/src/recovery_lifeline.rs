//! Recovery-lifeline dispatch (M8A-1, evidence-only).
//!
//! A dedicated, minimal dispatch path checked BEFORE the general `AGENT_METHODS`
//! table so the lifeline is provably separate: it renders the pinned command table
//! and returns typed `capability_denied` for every not-yet-implemented endpoint,
//! and it imports NONE of the wasm / provider / network / TLS machinery. The frozen
//! table + its `lifeline_vocabulary_sha256` fingerprint live in
//! `raios_core::recovery_lifeline_table` (host-tested). This slice grants nothing:
//! only `recovery.lifeline_table` is implemented (a pure table read); the five spec
//! endpoints deny. Honest posture only — `dev_key_not_owner_sealed`, owner_sealed
//! false, mutates nothing, current_boot.

use alloc::vec;

use raios_core::record::Value;
use raios_core::recovery_lifeline_table as table;

use super::DispatchOutcome;
use crate::agent_protocol_support::{
    begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
    record_str as s,
};

/// Lifeline dispatch. Returns `Some(outcome)` for any pinned lifeline method and
/// `None` for everything else so the general dispatcher runs unchanged. The lifeline
/// is intentionally the FIRST check in `agent_protocol::dispatch`.
pub(crate) fn dispatch(method: &str) -> Option<DispatchOutcome> {
    let matched = table::lookup(method)?;
    if matched.implemented {
        emit_table(matched.name);
        Some(DispatchOutcome::Response(matched.name))
    } else {
        emit_denied(matched);
        Some(DispatchOutcome::Denied(matched.name))
    }
}

fn emit_table(method: &'static str) {
    begin_response(method);
    // Render the exact logical table that the fingerprint is computed over, then
    // append the fingerprint itself (it is not part of what it hashes).
    let mut fields = table::build_core_fields();
    fields.push(f(
        "lifeline_vocabulary_sha256",
        Value::Sha256(table::vocabulary_sha256()),
    ));
    emit_record_fields(fields, 6);
    end_response(method);
}

fn emit_denied(matched: &'static table::LifelineMethod) {
    let reason = if matched.mutating {
        "recovery_mutation_not_implemented_this_boot"
    } else {
        "recovery_read_not_implemented_this_boot"
    };
    begin_response(matched.name);
    emit_record_fields(
        vec![
            f("schema", s("raios.recovery.v0")),
            f("scope", s(table::SCOPE)),
            f("classification", s(table::CLASSIFICATION)),
            f("method", s(matched.name)),
            f("requested_capability", s(matched.capability)),
            f("status", s("capability_denied")),
            f("reason", s(reason)),
            f("implemented", b(matched.implemented)),
            f("mutating", b(matched.mutating)),
            f("mutates_state", b(false)),
            f("routes_through_wasm", b(false)),
            f("routes_through_provider", b(false)),
            f("transport", s(table::TRANSPORT)),
            f("trust_state", s(table::TRUST_STATE)),
            f("trust_tier", s(table::TRUST_TIER)),
            f("owner_sealed", b(false)),
        ],
        6,
    );
    end_response(matched.name);
}
