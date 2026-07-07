//! Recovery-lifeline dispatch (M8A-1 boundary, M8A-2 read-only snapshot).
//!
//! A dedicated dispatch path checked BEFORE the general `AGENT_METHODS` table so
//! the lifeline is provably separate. It renders the pinned command table and a
//! read-only recovery snapshot, and returns typed `capability_denied` for every
//! not-yet-implemented (mutating) endpoint. It imports NO wasm / TLS / network /
//! event-log / durable-write machinery, makes no provider CALL (it only reads
//! provider STATE for diagnosis), and mutates nothing. The frozen table + its
//! `lifeline_vocabulary_sha256` fingerprint live in
//! `raios_core::recovery_lifeline_table` (host-tested). Honest posture only:
//! `dev_key_not_owner_sealed`, owner_sealed false, current_boot.

use alloc::{vec, vec::Vec};

use raios_core::boot_control::BootPosture;
use raios_core::record::Value;
use raios_core::recovery_lifeline_table as table;

use super::boot_control;
use super::DispatchOutcome;
use crate::agent_protocol_support::{
    begin_response, emit_record_fields, end_response, record_bool as b, record_event_or_null,
    record_field as f, record_str as s,
};
use crate::{echo_service, provider, service_inventory, system_status, ui};

/// Lifeline dispatch. Returns `Some(outcome)` for any pinned lifeline method and
/// `None` for everything else so the general dispatcher runs unchanged. The lifeline
/// is intentionally the FIRST check in `agent_protocol::dispatch`.
pub(crate) fn dispatch(method: &str, runtime: ui::RuntimeStatus) -> Option<DispatchOutcome> {
    let matched = table::lookup(method)?;
    match matched.name {
        table::METHOD_LIFELINE_TABLE => {
            emit_table(matched.name);
            Some(DispatchOutcome::Response(matched.name))
        }
        table::METHOD_SNAPSHOT => {
            emit_snapshot(matched.name, runtime);
            Some(DispatchOutcome::Response(matched.name))
        }
        _ => {
            emit_denied(matched);
            Some(DispatchOutcome::Denied(matched.name))
        }
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

fn posture_str(posture: BootPosture) -> &'static str {
    match posture {
        BootPosture::Normal => "normal",
        BootPosture::Probation => "probation",
        BootPosture::Safe => "safe",
        BootPosture::PersistenceUnavailable => "persistence_unavailable",
    }
}

/// Read-only recovery snapshot: current boot posture + the live service inventory
/// (id/kind/core_owned/replaceable/health) so an operator can DIAGNOSE what is
/// broken before deciding to restore. Reads only; writes nothing; redacts secrets
/// (only structural facts and fixed-vocabulary health states are reported — the
/// free-form `last_error` detail is deliberately dropped so no wifi/TLS/OpenAI
/// text can leak). NOTE: `current_boot_posture()` performs a bounded, read-only
/// BOOTCTL read each call; if storage is wedged it degrades to
/// `persistence_unavailable` (never hangs) — that degraded value is itself a
/// diagnostic signal, so the dependency is intentional. A cached boot decision is
/// deferred to the M8C durable-last-good/SAFE integration.
fn emit_snapshot(method: &'static str, runtime: ui::RuntimeStatus) {
    let status = system_status::SystemSnapshot::collect(None, runtime);
    let provider_state = provider::snapshot();

    let mut rows: Vec<Value<'static>> = Vec::new();
    let mut core_owned_count = 0u64;
    let mut replaceable_count = 0u64;
    let mut unhealthy_count = 0u64;
    let mut idx = 0usize;
    while idx < service_inventory::SERVICES.len() {
        let svc = &service_inventory::SERVICES[idx];
        let health = service_inventory::service_health(svc, &status, &provider_state);
        if svc.core_owned {
            core_owned_count += 1;
        }
        if svc.replaceable {
            replaceable_count += 1;
        }
        if health.state != "healthy" && health.state != "starting" {
            unhealthy_count += 1;
        }
        rows.push(Value::InlineObject(vec![
            f("id", s(svc.id)),
            f("kind", s(svc.kind)),
            f("core_owned", b(svc.core_owned)),
            f("replaceable", b(svc.replaceable)),
            f("health", s(health.state)),
        ]));
        idx += 1;
    }

    // Crashed services come from each service's OWN live crash bookkeeping, not from
    // the static `service_inventory::SERVICES` table (echo is a RAM-only current-boot
    // service and is not a member of it). This is a plain read-only bool/id read that
    // MUST NOT invoke wasm — `echo_service::crash_view()` only inspects the crash flag.
    let mut crashed_services: Vec<Value<'static>> = Vec::new();
    if let Some((id, health, last_error_id)) = echo_service::crash_view() {
        crashed_services.push(Value::InlineObject(vec![
            f("id", s(id)),
            f("health", s(health)),
            f("last_error_id", record_event_or_null(last_error_id)),
        ]));
    }
    let crashed_service_count = crashed_services.len() as u64;

    begin_response(method);
    emit_record_fields(
        vec![
            f("schema", s(table::SNAPSHOT_SCHEMA)),
            f("scope", s(table::SCOPE)),
            f("classification", s(table::CLASSIFICATION)),
            f("transport", s(table::TRANSPORT)),
            f("trust_state", s(table::TRUST_STATE)),
            f("trust_tier", s(table::TRUST_TIER)),
            f(
                "boot_posture",
                s(posture_str(boot_control::current_boot_posture())),
            ),
            f("mutates_state", b(false)),
            f("owner_sealed", b(false)),
            f("routes_through_wasm", b(false)),
            f("routes_through_provider", b(false)),
            f("redacted", b(true)),
            f("lifeline_available", b(true)),
            f("lifeline_table_method", s(table::METHOD_LIFELINE_TABLE)),
            f(
                "service_count",
                Value::U64(service_inventory::SERVICES.len() as u64),
            ),
            f("core_owned_count", Value::U64(core_owned_count)),
            f("replaceable_count", Value::U64(replaceable_count)),
            f("unhealthy_count", Value::U64(unhealthy_count)),
            f("crashed_service_count", Value::U64(crashed_service_count)),
            f("services", Value::Array(rows)),
            f("crashed_services", Value::Array(crashed_services)),
        ],
        6,
    );
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
