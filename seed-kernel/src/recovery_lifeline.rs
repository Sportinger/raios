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

use raios_core::artifact_blob_frame::build_artifact_blob_frame;
use raios_core::boot_control::BootPosture;
use raios_core::promotion_attestation::PROMOTION_AUTHORITY_IS_PLACEHOLDER;
use raios_core::record::Value;
use raios_core::recovery_lifeline_table as table;
use raios_core::recovery_load_record as load_rec;
use raios_core::repromotion_reverify::{reverify_persisted_artifact, RepromotionReverifyInput};
use raios_core::sha256_bytes;

use super::{artifact_store, boot_control, durable_store, repromotion, DispatchOutcome};
use crate::agent_protocol_support::{
    begin_response, emit_record_fields, end_response, record_bool as b, record_event_or_null,
    record_field as f, record_sha_or_null, record_str as s,
};
use crate::{echo_service, event_log, pci, provider, service_inventory, system_status, ui};

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
    // `recovery.restart_last_good <target>` — same head-token discipline as disable, so
    // `recovery.restart_last_good_target_binding_diagnostic` (rest starts with `_`, not
    // whitespace) still falls through to the general table rather than this executor.
    if raios_core::method_head_eq(method, table::METHOD_RESTART_LAST_GOOD) {
        let rest = method[table::METHOD_RESTART_LAST_GOOD.len()..].trim();
        return Some(emit_restart_last_good(rest));
    }
    // `recovery.load_artifact_by_hash <sha256>` — same head-token discipline: the frozen
    // `recovery.load_artifact_by_hash_target_binding_diagnostic` (rest starts with `_`, not
    // whitespace) still falls through to the general table rather than this executor, and
    // the no-arg form remains reachable (it denies `malformed_hash`).
    if raios_core::method_head_eq(method, table::METHOD_LOAD_ARTIFACT_BY_HASH) {
        let rest = method[table::METHOD_LOAD_ARTIFACT_BY_HASH.len()..].trim();
        return Some(emit_load_artifact_by_hash(rest));
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
/// text can leak). NOTE: `current_boot_last_good_view()` performs ONE bounded,
/// read-only BOOTCTL read each call, feeding boot_posture AND the M8C-1
/// durable_last_good/rollback_preview projections from a single decision (no skew);
/// if storage is wedged it degrades to `persistence_unavailable` (never hangs) with
/// honest missing-evidence — that degraded value is itself a diagnostic signal, so
/// the dependency is intentional.
fn emit_snapshot(method: &'static str, runtime: ui::RuntimeStatus) {
    let status = system_status::SystemSnapshot::collect(None, runtime);
    let provider_state = provider::snapshot();
    // ONE read-only bootctl read feeds the boot_posture field AND the two M8C-1
    // durable projections below (durable_last_good + rollback_preview), so posture,
    // decision, and the authoritative record never skew across separate reads. This
    // read mutates nothing, appends nothing, and re-pins no vocabulary.
    let (boot_posture, boot_decision, boot_control_record) =
        boot_control::current_boot_last_good_view();

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
            f("boot_posture", s(posture_str(boot_posture))),
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
            // M8C-1: read-only durable last-good pointer + rollback-to-last-good
            // projection, both derived from the SINGLE bootctl read above. They mutate
            // nothing, add no lifeline method, re-pin no vocabulary, and fail honest
            // (available:false, boot_success_mark:"missing", null slots) when no
            // authoritative record exists. rollback_preview is a pure projection:
            // recovery.rollback stays denied (mutating_rollback_available_via_lifeline
            // is false; the mutating pointer switch remains M7C's authority).
            f(
                "durable_last_good",
                Value::Object(raios_core::boot_control::recovery_last_good_fields(
                    &boot_decision,
                    boot_control_record.as_ref(),
                )),
            ),
            f(
                "rollback_preview",
                Value::Object(raios_core::boot_control::recovery_rollback_preview_fields(
                    &boot_decision,
                    boot_control_record.as_ref(),
                )),
            ),
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
        // Restart-only fields: unused by the disable payload/evaluation (kept false).
        restores_known_good: false,
        restart_target_restorable: false,
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

/// `recovery.restart_last_good <target>` executor (the mutating half). Classifies the
/// target read-only and reads echo's live restorability, denies BEFORE mutate AND
/// BEFORE append for SAFE posture / every invalid target class / a not-restorable
/// target, otherwise appends a durable `raios.recovery_action.v0` record through the
/// SHARED reclog gauntlet (authorized ONLY by the 2a evaluator) and — ONLY if that
/// append is `performed` — clears echo's disabled+crashed latches and re-runs the
/// EXISTING verified start path back to healthy/running. Restore-only; grants nothing;
/// restores a built-in module already in RAM (no persistence, no new bytes).
fn emit_restart_last_good(arg: &str) -> DispatchOutcome {
    let arg = arg.trim();
    let is_lifeline = target_is_lifeline_endpoint(arg);
    let core_owned = target_is_core_owned(arg);
    let known = echo_service::disable_target_matches(arg);
    let restorable = echo_service::restart_restorable();
    // The durable record attests the AUTHORIZED ACTION (it is written before the re-run, as
    // append-before-mutate requires), NOT the post-run health: `mutates_live_state`/
    // `restores_known_good` describe the authorized restart intent. The live outcome
    // (`running`/`health`/`restarted_to_running`) is reported ONLY in the response from the
    // real `start()` result, so a failed re-run is never falsely reported as healthy.
    let record = durable_store::RecoveryActionRecord {
        action_kind: durable_store::RECOVERY_ACTION_KIND_RESTART_LAST_GOOD,
        disable_target_id: echo_service::ECHO_SERVICE_DESCRIPTOR.service_id,
        disable_target_is_lifeline_endpoint: is_lifeline,
        disable_target_core_owned: core_owned,
        disable_target_known_disablable: known,
        grants_new_capability: false,
        restores_known_good: true,
        restart_target_restorable: restorable,
    };

    // Deny-before-mutate AND deny-before-append: SAFE posture, every invalid-target
    // class, and a not-restorable target are rejected BEFORE any plan/write. On denial
    // nothing is restarted and nothing is written.
    if let Some(reason) = durable_store::recovery_restart_preflight_denial(
        boot_control::current_boot_posture(),
        record.classification(),
        restorable,
    ) {
        let evidence = durable_store::recovery_action_append_denied(&record, reason);
        emit_restart_response(arg, &evidence, None, None, false);
        return DispatchOutcome::Denied(table::METHOD_RESTART_LAST_GOOD);
    }

    // Durable recovery_action record FIRST, authorized only via the 2a evaluator.
    let evidence = durable_store::append_recovery_action(&record);
    if evidence.performed {
        // Only a durably-recorded, readback + reparse-verified authorization mutates
        // live state: clear the latches and re-run the verified start path.
        let (event_id, health, running) = echo_service::restart_last_good();
        emit_restart_response(arg, &evidence, Some(event_id), Some(health), running);
        DispatchOutcome::Response(table::METHOD_RESTART_LAST_GOOD)
    } else {
        emit_restart_response(arg, &evidence, None, None, false);
        DispatchOutcome::Denied(table::METHOD_RESTART_LAST_GOOD)
    }
}

/// Render the `raios.recovery_action.v0` result for a restart_last_good decision.
/// Mirrors the disable response but carries the restart-only outcome: `restores_known_good`,
/// `restarted_to_running`, and the resulting `health`/`running` from the re-run start
/// path. `mutates_live_state` is exactly `performed`: every denial mutates nothing.
fn emit_restart_response(
    requested_target: &str,
    evidence: &durable_store::RecoveryActionAppendEvidence,
    restart_event_id: Option<event_log::EventId>,
    restart_health: Option<&'static str>,
    restarted_to_running: bool,
) {
    let method = table::METHOD_RESTART_LAST_GOOD;
    let performed = evidence.performed;
    let disable_target_id = if evidence.disable_target_known_disablable {
        s(evidence.disable_target_id)
    } else {
        Value::Null
    };
    let health = match restart_health {
        Some(health) => s(health),
        None => Value::Null,
    };
    begin_response(method);
    emit_record_fields(
        vec![
            f("schema", s(durable_store::RECOVERY_ACTION_SCHEMA)),
            f("id", s("recovery_action.current_boot.restart_last_good.v0")),
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
            f("restores_known_good", b(evidence.restores_known_good)),
            f("restarted_to_running", b(restarted_to_running)),
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
            f("restart_event_id", record_event_or_null(restart_event_id)),
            f("health", health),
            f("running", b(restarted_to_running)),
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

// ===================================================================================
// M8D-1: recovery.load_artifact_by_hash — hash-selected FULL M6 re-verify, GRANTS NOTHING.
//
// The operator names an exact content hash; the lifeline parses it, selects the matching
// artifact_persist record from the LOCAL M7D store, and RE-VERIFIES the full M6 evidence
// chain from scratch by reusing the ONE shared reverify implementation
// (`repromotion::reverify_record_only`, which re-runs `reverify_persisted_artifact` and
// NEVER trusts a stored boolean), then REPORTS the decision. It grants nothing: no retain,
// no module load, no emit_load/emit_start, `authorizes_load`/`cross_reboot_proven` always
// false, and NO durable write. Fail-closed order: malformed hash (reads NOTHING) -> SAFE/
// PersistenceUnavailable posture (BEFORE any store read) -> AHCI controller absent ->
// record absent -> reverify. The positive LOAD path is M8D-2 (two-boot harness), not here.
// ===================================================================================

/// `recovery.load_artifact_by_hash <sha256>` executor. Reverify-ONLY; grants nothing.
fn emit_load_artifact_by_hash(arg: &str) -> DispatchOutcome {
    let arg = arg.trim();
    // 1. Parse the caller-supplied hash. `parse_sha256_ref` eats an optional `sha256:`
    //    prefix; empty/short/non-hex all fail. A malformed hash reads NOTHING.
    let Some(requested_hash) = raios_core::parse_sha256_ref(arg) else {
        emit_load_denied(arg, None, load_rec::REASON_MALFORMED_HASH);
        return DispatchOutcome::Denied(table::METHOD_LOAD_ARTIFACT_BY_HASH);
    };
    // 2. SAFE / PersistenceUnavailable posture -> deny BEFORE any store read.
    if let Some(reason) = load_by_hash_posture_denial(boot_control::current_boot_posture()) {
        emit_load_denied(arg, Some(requested_hash), reason);
        return DispatchOutcome::Denied(table::METHOD_LOAD_ARTIFACT_BY_HASH);
    }
    // 3. The local store lives on the AHCI mass-storage controller; absent -> deny.
    let Some(controller) = pci::find_mass_storage_controller() else {
        emit_load_denied(
            arg,
            Some(requested_hash),
            load_rec::REASON_CONTROLLER_ABSENT,
        );
        return DispatchOutcome::Denied(table::METHOD_LOAD_ARTIFACT_BY_HASH);
    };
    // 4. Select the artifact_persist record whose artifact_sha256 == the requested hash.
    let scan = artifact_store::current_boot_reclog_scan(controller);
    let records = scan
        .bytes
        .as_deref()
        .map(artifact_store::artifact_persist_records_from_reclog)
        .unwrap_or_default();
    let Some(record) = select_record_by_hash(&records, requested_hash) else {
        emit_load_denied(
            arg,
            Some(requested_hash),
            load_rec::REASON_NOT_IN_LOCAL_STORE,
        );
        return DispatchOutcome::Denied(table::METHOD_LOAD_ARTIFACT_BY_HASH);
    };
    // 5. RE-VERIFY the full M6 chain from scratch via the ONE shared implementation.
    //    GRANTS NOTHING: `reverify_record_only` retains/loads/starts nothing and writes
    //    nothing durable. We only REPORT its decision here (the positive load is M8D-2).
    let outcome = repromotion::reverify_record_only(controller, &record);
    emit_load_reverify_response(arg, requested_hash, &record, &outcome);
    if outcome.reverified() {
        DispatchOutcome::Response(table::METHOD_LOAD_ARTIFACT_BY_HASH)
    } else {
        DispatchOutcome::Denied(table::METHOD_LOAD_ARTIFACT_BY_HASH)
    }
}

/// SAFE / PersistenceUnavailable both deny before any store read; a writable posture
/// (Normal/Probation) proceeds to the store select + reverify.
fn load_by_hash_posture_denial(posture: BootPosture) -> Option<&'static str> {
    match posture {
        BootPosture::Safe | BootPosture::PersistenceUnavailable => Some(load_rec::REASON_SAFE_MODE),
        BootPosture::Normal | BootPosture::Probation => None,
    }
}

/// Exact-hash select over the locally-stored artifact_persist records (no fuzzy match).
fn select_record_by_hash(
    records: &[artifact_store::ArtifactPersistRecord],
    hash: [u8; 32],
) -> Option<artifact_store::ArtifactPersistRecord> {
    let mut idx = 0usize;
    while idx < records.len() {
        if records[idx].artifact_sha256 == hash {
            return Some(records[idx]);
        }
        idx += 1;
    }
    None
}

/// Pre-reverify denial (malformed / SAFE / controller-absent / not-in-store): nothing was
/// read past the failing gate and nothing was reverified.
fn emit_load_denied(arg: &str, requested_hash: Option<[u8; 32]>, reason: &'static str) {
    emit_load_response(
        arg,
        requested_hash,
        load_rec::STATUS_DENIED,
        reason,
        false,
        false,
        "not_attempted",
        load_rec::REVERIFY_NOT_ATTEMPTED,
        None,
        None,
        None,
        None,
    );
}

/// Reverify result: the chain either re-verified (load STILL denied — M8D-1 grants
/// nothing) or was denied with the reverify reason. Loads nothing either way.
fn emit_load_reverify_response(
    arg: &str,
    requested_hash: [u8; 32],
    record: &artifact_store::ArtifactPersistRecord,
    outcome: &repromotion::ReverifyOnly,
) {
    let reverified = outcome.reverified();
    let (status, reason) = if reverified {
        (
            load_rec::STATUS_REVERIFIED_LOAD_DENIED,
            load_rec::REASON_REVERIFIED_LOAD_DENIED,
        )
    } else {
        (load_rec::STATUS_DENIED, outcome.reason())
    };
    emit_load_response(
        arg,
        Some(requested_hash),
        status,
        reason,
        true,
        reverified,
        outcome.status(),
        outcome.reason(),
        Some(record),
        outcome.candidate_payload_sha256(),
        outcome.recomputed_attestation_reference_hash(),
        outcome.recorded_attestation_reference_hash(),
    );
}

#[allow(clippy::too_many_arguments)]
fn emit_load_response(
    requested_target: &str,
    requested_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    reverify_attempted: bool,
    reverified: bool,
    reverify_status: &'static str,
    reverify_reason: &'static str,
    record: Option<&artifact_store::ArtifactPersistRecord>,
    candidate_payload_sha256: Option<[u8; 32]>,
    recomputed_attestation_reference_hash: Option<[u8; 32]>,
    recorded_attestation_reference_hash: Option<[u8; 32]>,
) {
    let method = table::METHOD_LOAD_ARTIFACT_BY_HASH;
    let mut fields = vec![
        f("schema", s(load_rec::SCHEMA)),
        f("id", s(load_rec::RESPONSE_ID)),
        f("scope", s(load_rec::SCOPE)),
        f("classification", s(load_rec::CLASSIFICATION)),
        f("method", s(method)),
        f("requested_target", s(requested_target)),
        f("requested_hash", record_sha_or_null(requested_hash)),
        f("requested_hash_valid", b(requested_hash.is_some())),
        f("status", s(status)),
        f("reason", s(reason)),
        f("reverify_attempted", b(reverify_attempted)),
        f("reverified", b(reverified)),
        f("reverify_status", s(reverify_status)),
        f("reverify_reason", s(reverify_reason)),
        f("record_found", b(record.is_some())),
        f("record_seq", optional_u64(record.map(|record| record.seq))),
        f(
            "artifact_sha256",
            record_sha_or_null(record.map(|record| record.artifact_sha256)),
        ),
        f(
            "manifest_hash",
            record_sha_or_null(record.map(|record| record.manifest_hash)),
        ),
        f(
            "vm_report_hash",
            record_sha_or_null(record.map(|record| record.vm_report_hash)),
        ),
        f(
            "grant_hash",
            record_sha_or_null(record.map(|record| record.grant_hash)),
        ),
        f(
            "promotion_transaction_sha256",
            record_sha_or_null(record.map(|record| record.promotion_transaction_sha256)),
        ),
        f(
            "artstor_blob_offset",
            optional_u64(record.map(|record| record.artstor_blob_offset)),
        ),
        f(
            "artstor_blob_len",
            optional_u64(record.map(|record| record.artstor_blob_len)),
        ),
        f(
            "artstor_blob_frame_sha256",
            record_sha_or_null(record.map(|record| record.artstor_blob_frame_sha256)),
        ),
        f(
            "candidate_payload_sha256",
            record_sha_or_null(candidate_payload_sha256),
        ),
        f(
            "recomputed_attestation_reference_hash",
            record_sha_or_null(recomputed_attestation_reference_hash),
        ),
        f(
            "recorded_attestation_reference_hash",
            record_sha_or_null(recorded_attestation_reference_hash),
        ),
        f("transport", s(table::TRANSPORT)),
        f("trust_state", s(table::TRUST_STATE)),
        f("store_source", s("local_recovery_store")),
    ];
    for field in load_rec::grants_nothing_fields() {
        fields.push(field);
    }
    begin_response(method);
    emit_record_fields(fields, 6);
    end_response(method);
}

/// Kernel-side denial-truth-table selftest for `recovery.load_artifact_by_hash` (mirrors
/// `durable_store::emit_recovery_action_selftest`). Pure: it reads no disk and mutates
/// nothing. It exercises malformed / SAFE / PersistenceUnavailable / absent-store, and —
/// via the SAME pure `reverify_persisted_artifact` decision the executor uses — two
/// synthetic reverify denials (blob-hash mismatch, missing promotion transaction) so a
/// re-verify that does not bind can never masquerade as loadable.
pub(crate) fn emit_load_artifact_by_hash_selftest() {
    let cases = load_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);
    let case_records: Vec<Value<'static>> = cases.iter().map(load_selftest_case_record).collect();

    begin_response(load_rec::SELFTEST_METHOD);
    let mut fields = vec![
        f("schema", s(load_rec::SELFTEST_SCHEMA)),
        f("scope", s(load_rec::SCOPE)),
        f("classification", s(load_rec::CLASSIFICATION)),
        f("method", s(load_rec::SELFTEST_METHOD)),
        f("test_infrastructure", b(true)),
        f("mutates_global_event_log", b(false)),
        f("writes_persistent_state", b(false)),
        f("loads_artifact", b(false)),
        f("reads_local_store", b(false)),
        f("case_count", Value::U64(cases.len() as u64)),
        f("passed", b(passed)),
        f("cases", Value::Array(case_records)),
        f("evidence_complete", b(true)),
    ];
    for field in load_rec::grants_nothing_fields() {
        fields.push(field);
    }
    emit_record_fields(fields, 6);
    end_response(load_rec::SELFTEST_METHOD);
}

#[derive(Clone, Copy)]
struct LoadSelftestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

fn load_selftest_cases() -> Vec<LoadSelftestCase> {
    vec![
        load_parse_denial_case("malformed_hash_empty_denied", ""),
        load_parse_denial_case(
            "malformed_hash_non_hex_denied",
            "sha256:zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
        ),
        load_posture_denial_case("safe_posture_denied", BootPosture::Safe),
        load_posture_denial_case(
            "persistence_unavailable_denied",
            BootPosture::PersistenceUnavailable,
        ),
        load_absent_store_case(),
        load_reverify_case("reverify_blob_mismatch_denied", "blob_hash_mismatch", true),
        load_reverify_case(
            "reverify_transaction_missing_denied",
            "promotion_transaction_not_found",
            false,
        ),
    ]
}

fn load_denial_case(
    name: &'static str,
    expected_reason: &'static str,
    actual_reason: &'static str,
) -> LoadSelftestCase {
    LoadSelftestCase {
        name,
        expected_status: load_rec::STATUS_DENIED,
        expected_reason,
        actual_status: load_rec::STATUS_DENIED,
        actual_reason,
        passed: actual_reason == expected_reason,
    }
}

fn load_parse_denial_case(name: &'static str, input: &str) -> LoadSelftestCase {
    let actual_reason = match raios_core::parse_sha256_ref(input) {
        Some(_) => "unexpected_parsed",
        None => load_rec::REASON_MALFORMED_HASH,
    };
    load_denial_case(name, load_rec::REASON_MALFORMED_HASH, actual_reason)
}

fn load_posture_denial_case(name: &'static str, posture: BootPosture) -> LoadSelftestCase {
    let actual_reason = load_by_hash_posture_denial(posture).unwrap_or("unexpected_writable");
    load_denial_case(name, load_rec::REASON_SAFE_MODE, actual_reason)
}

fn load_absent_store_case() -> LoadSelftestCase {
    let actual_reason = match select_record_by_hash(&[], [0x11; 32]) {
        Some(_) => "unexpected_record_found",
        None => load_rec::REASON_NOT_IN_LOCAL_STORE,
    };
    load_denial_case(
        "absent_store_denied",
        load_rec::REASON_NOT_IN_LOCAL_STORE,
        actual_reason,
    )
}

/// Drive the SAME pure `reverify_persisted_artifact` decision the executor uses over a
/// synthetic input, proving a re-verify that does not bind fails closed (never trusts a
/// stored boolean). `corrupt_blob_hash` selects the failure: a mismatched blob-frame hash
/// (`blob_hash_mismatch`) vs a well-formed blob with no promotion transaction
/// (`promotion_transaction_not_found`).
fn load_reverify_case(
    name: &'static str,
    expected_reason: &'static str,
    corrupt_blob_hash: bool,
) -> LoadSelftestCase {
    let payload = b"m8d1 recovery_load selftest payload";
    let blob = build_artifact_blob_frame(payload);
    let record_artstor_blob_frame_sha256 = if corrupt_blob_hash {
        [0u8; 32]
    } else {
        sha256_bytes(&blob)
    };
    let decision = reverify_persisted_artifact(&RepromotionReverifyInput {
        artstor_blob_bytes: &blob,
        record_artstor_blob_frame_sha256,
        record_artifact_sha256: sha256_bytes(payload),
        record_manifest_hash: [0u8; 32],
        record_vm_report_hash: [0u8; 32],
        record_grant_hash: [0u8; 32],
        transaction: None,
        recomputed_attestation_reference_hash: [0u8; 32],
        promotion_authority_is_placeholder: PROMOTION_AUTHORITY_IS_PLACEHOLDER,
    });
    let actual_status = if decision.reverified {
        "unexpected_reverified"
    } else {
        load_rec::STATUS_DENIED
    };
    LoadSelftestCase {
        name,
        expected_status: load_rec::STATUS_DENIED,
        expected_reason,
        actual_status,
        actual_reason: decision.reason,
        passed: !decision.reverified && decision.reason == expected_reason,
    }
}

fn load_selftest_case_record(case: &LoadSelftestCase) -> Value<'static> {
    Value::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f("passed", b(case.passed)),
        f("loads_artifact", b(false)),
        f("mutates_live_state", b(false)),
    ])
}
