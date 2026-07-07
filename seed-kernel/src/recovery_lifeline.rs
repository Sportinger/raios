//! Recovery-lifeline dispatch (M8A-1 boundary, M8A-2 read-only snapshot, M8B-1
//! `recovery.disable_module` current-boot executor).
//!
//! A dedicated dispatch path checked BEFORE the general `AGENT_METHODS` table so
//! the lifeline is provably separate. It renders the pinned command table, a
//! read-only recovery snapshot, and the ONE bounded mutating executor implemented so
//! far — `recovery.disable_module`, which stops+disables the single non-core
//! current-boot module `svc.demo.echo` ONLY after a durable `raios.recovery_action.v0`
//! record has been appended, read back, and reparse-verified through the shared reclog
//! mechanism (authorized solely by `evaluate_scoped_recovery_action_append`). Every
//! other mutating endpoint still returns typed `capability_denied`. Deny-before-mutate
//! is enforced on every rule: SAFE posture and invalid targets (core-owned, the
//! lifeline endpoint itself, unknown/`*`/`all`) are rejected before any plan/write, so
//! nothing is disabled and nothing is written on denial. The disable is current-boot
//! RAM-only (`persistence_claimed:false`, `owner_sealed:false`, grants no new
//! capability). Aside from the disable latch it imports NO wasm / TLS / network
//! machinery and makes no provider CALL (it only reads provider STATE for diagnosis).
//! The frozen table + its `lifeline_vocabulary_sha256` fingerprint live in
//! `raios_core::recovery_lifeline_table` (host-tested). Honest posture only:
//! `dev_key_not_owner_sealed`, owner_sealed false, current_boot.

use alloc::{vec, vec::Vec};

use raios_core::boot_control::BootPosture;
use raios_core::record::Value;
use raios_core::recovery_lifeline_table as table;

use super::{boot_control, durable_store, DispatchOutcome};
use crate::agent_protocol_support::{
    begin_response, emit_record_fields, end_response, record_bool as b, record_event_or_null,
    record_field as f, record_sha_or_null, record_str as s,
};
use crate::{echo_service, event_log, provider, service_inventory, system_status, ui};

/// Lifeline dispatch. Returns `Some(outcome)` for any pinned lifeline method and
/// `None` for everything else so the general dispatcher runs unchanged. The lifeline
/// is intentionally the FIRST check in `agent_protocol::dispatch`.
pub(crate) fn dispatch(method: &str, runtime: ui::RuntimeStatus) -> Option<DispatchOutcome> {
    // `recovery.disable_module` takes a target argument, so it must be matched on the
    // method HEAD (not the full-string `table::lookup`) and BEFORE it — otherwise the
    // exact-lookup path would route the no-arg form to the generic denial. Everything
    // after the head token is the (untrusted) target id, classified read-only below.
    if raios_core::method_head_eq(method, table::METHOD_DISABLE_MODULE) {
        let rest = method[table::METHOD_DISABLE_MODULE.len()..].trim();
        return Some(emit_disable_module(rest));
    }
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

    // Disabled modules come from each module's OWN live disable latch (same read-only,
    // no-wasm discipline as crashed_services). `echo_service::disabled_view()` only
    // inspects the current-boot `disabled` flag set by `recovery.disable_module`.
    let mut disabled_modules: Vec<Value<'static>> = Vec::new();
    if let Some((id, health, disable_event_id)) = echo_service::disabled_view() {
        disabled_modules.push(Value::InlineObject(vec![
            f("id", s(id)),
            f("health", s(health)),
            f("disable_event_id", record_event_or_null(disable_event_id)),
        ]));
    }
    let disabled_module_count = disabled_modules.len() as u64;

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
            f("disabled_module_count", Value::U64(disabled_module_count)),
            f("services", Value::Array(rows)),
            f("crashed_services", Value::Array(crashed_services)),
            f("disabled_modules", Value::Array(disabled_modules)),
        ],
        6,
    );
    end_response(method);
}

/// `recovery.disable_module <target>` executor (the mutating half). Classifies the
/// target read-only, denies before mutate for SAFE posture / invalid target classes,
/// otherwise appends a durable `raios.recovery_action.v0` record through the shared
/// reclog gauntlet (scan -> plan -> write -> readback -> reparse -> evaluate -> rescan,
/// authorized ONLY by the 1a evaluator) and — ONLY if that append is `performed` —
/// stops+disables `svc.demo.echo` for the current boot. Restore-only; grants nothing.
fn emit_disable_module(arg: &str) -> DispatchOutcome {
    let arg = arg.trim();
    let is_lifeline = target_is_lifeline_endpoint(arg);
    let core_owned = target_is_core_owned(arg);
    let known = echo_service::disable_target_matches(arg);
    let record = durable_store::RecoveryActionRecord {
        action_kind: durable_store::RECOVERY_ACTION_KIND_DISABLE_MODULE,
        // Canonical id; only rendered/used when the target is actually known-disablable.
        disable_target_id: echo_service::ECHO_SERVICE_DESCRIPTOR.service_id,
        disable_target_is_lifeline_endpoint: is_lifeline,
        disable_target_core_owned: core_owned,
        disable_target_known_disablable: known,
        grants_new_capability: false,
    };

    // Deny-before-mutate: SAFE posture and every invalid-target class are rejected
    // BEFORE any plan/write. On denial nothing is disabled and nothing is written.
    if let Some(reason) = durable_store::recovery_action_preflight_denial(
        boot_control::current_boot_posture(),
        record.classification(),
    ) {
        let evidence = durable_store::recovery_action_append_denied(&record, reason);
        emit_recovery_action_response(arg, &evidence, None);
        return DispatchOutcome::Denied(table::METHOD_DISABLE_MODULE);
    }

    // Durable recovery_action record FIRST, authorized only via the 1a evaluator.
    let evidence = durable_store::append_recovery_action(&record);
    if evidence.performed {
        // Only a durably-recorded, readback + reparse-verified authorization mutates
        // live state: stop+disable echo for the current boot.
        let disable_event_id = echo_service::disable();
        emit_recovery_action_response(arg, &evidence, Some(disable_event_id));
        DispatchOutcome::Response(table::METHOD_DISABLE_MODULE)
    } else {
        emit_recovery_action_response(arg, &evidence, None);
        DispatchOutcome::Denied(table::METHOD_DISABLE_MODULE)
    }
}

/// A target is the lifeline endpoint itself when it matches ANY of the six pinned
/// lifeline method names (case-insensitive). Disabling the lifeline is denied.
fn target_is_lifeline_endpoint(arg: &str) -> bool {
    let mut idx = 0usize;
    while idx < table::LIFELINE_METHODS.len() {
        if raios_core::method_eq(arg, table::LIFELINE_METHODS[idx].name) {
            return true;
        }
        idx += 1;
    }
    false
}

/// A target is core-owned when it matches a `SERVICES` id whose `core_owned` is true.
/// Core services can never be disabled through the lifeline.
fn target_is_core_owned(arg: &str) -> bool {
    let mut idx = 0usize;
    while idx < service_inventory::SERVICES.len() {
        let svc = &service_inventory::SERVICES[idx];
        if svc.core_owned && raios_core::method_eq(arg, svc.id) {
            return true;
        }
        idx += 1;
    }
    false
}

fn optional_u64(value: Option<u64>) -> Value<'static> {
    match value {
        Some(value) => Value::U64(value),
        None => Value::Null,
    }
}

/// Render the `raios.recovery_action.v0` result for a disable_module decision. Carries
/// the durable-append evidence, the read-only target classification, and — on the
/// authorized path — the disable lifecycle event id. `mutates_live_state` is exactly
/// `performed`: every denial mutates nothing.
fn emit_recovery_action_response(
    requested_target: &str,
    evidence: &durable_store::RecoveryActionAppendEvidence,
    disable_event_id: Option<event_log::EventId>,
) {
    let method = table::METHOD_DISABLE_MODULE;
    let performed = evidence.performed;
    let disable_target_id = if evidence.disable_target_known_disablable {
        s(evidence.disable_target_id)
    } else {
        Value::Null
    };
    let health = if performed {
        s(echo_service::HEALTH_DISABLED)
    } else {
        Value::Null
    };
    begin_response(method);
    emit_record_fields(
        vec![
            f("schema", s(durable_store::RECOVERY_ACTION_SCHEMA)),
            f("id", s("recovery_action.current_boot.disable_module.v0")),
            f("scope", s(table::SCOPE)),
            f("classification", s(table::CLASSIFICATION)),
            f("method", s(method)),
            f("action_kind", s(evidence.action_kind)),
            f("requested_target", s(requested_target)),
            f("disable_target_id", disable_target_id),
            f(
                "disable_target_is_lifeline_endpoint",
                b(evidence.disable_target_is_lifeline_endpoint),
            ),
            f(
                "disable_target_core_owned",
                b(evidence.disable_target_core_owned),
            ),
            f(
                "disable_target_known_disablable",
                b(evidence.disable_target_known_disablable),
            ),
            f(
                "status",
                s(if performed { "ok" } else { "capability_denied" }),
            ),
            f("durable_append", s(evidence.durable_append)),
            f("performed", b(performed)),
            f("reason", s(evidence.reason)),
            f("authority", s(evidence.authority)),
            f(
                "target_id",
                s(durable_store::RECOVERY_ACTION_APPEND_TARGET_ID),
            ),
            f("record_schema", s(durable_store::RECOVERY_ACTION_SCHEMA)),
            f(
                "region_marker",
                s(durable_store::RECOVERY_ACTION_APPEND_REGION_MARKER),
            ),
            f("mutates_live_state", b(performed)),
            f("reversible_this_boot", b(false)),
            f("grants_new_capability", b(evidence.grants_new_capability)),
            f("owner_sealed", b(evidence.owner_sealed)),
            f("persistence_claimed", b(evidence.persistence_claimed)),
            f(
                "promotion_authority_is_placeholder",
                b(evidence.promotion_authority_is_placeholder),
            ),
            f("trust_tier", s(evidence.trust_tier)),
            f("transport", s(table::TRANSPORT)),
            f("trust_state", s(table::TRUST_STATE)),
            f("routes_through_wasm", b(false)),
            f("routes_through_provider", b(false)),
            f("seq", optional_u64(evidence.seq)),
            f("write_offset", optional_u64(evidence.write_offset)),
            f("frame_len", optional_u64(evidence.frame_len)),
            f(
                "payload_sha256",
                record_sha_or_null(evidence.payload_sha256),
            ),
            f("frame_sha256", record_sha_or_null(evidence.frame_sha256)),
            f(
                "readback_sha256",
                record_sha_or_null(evidence.readback_sha256),
            ),
            f("reparse_valid", b(evidence.reparse_valid)),
            f("tail_seq_before", optional_u64(evidence.tail_seq_before)),
            f("count_before", optional_u64(evidence.count_before)),
            f("tail_seq_after", optional_u64(evidence.tail_seq_after)),
            f("count_after", optional_u64(evidence.count_after)),
            f("disable_event_id", record_event_or_null(disable_event_id)),
            f("health", health),
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
