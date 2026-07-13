use alloc::{vec, vec::Vec};

use super::*;
use crate::agent_protocol_support::emit_record_fields;
use crate::current_boot_service::ServiceState;
use raios_core::record::{Field, Value as V};

macro_rules! j {
    ($key:literal => $value:expr) => {
        Field::new($key, $value)
    };
}

pub(crate) fn artifact_identity_signature_verified(descriptor: LoadDescriptor) -> bool {
    let identity = descriptor.artifact_identity;
    descriptor_sources::verify_artifact_identity_envelope_parts(
        identity.signed_envelope,
        identity.id,
        identity.artifact_id,
        identity.text,
    )
}

#[derive(Clone, Copy)]
pub(crate) struct Snapshot {
    pub loaded: bool,
    pub running: bool,
    pub generation: u64,
    pub state_counter: u64,
    pub state_migration: Option<HelloStateMigrationRecord>,
    pub hot_swap_probation: Option<HelloHotSwapProbationRecord>,
    pub applied_rollback: Option<AppliedRollbackRecord>,
    pub load_descriptor: LoadDescriptor,
    pub last_action: &'static str,
    pub last_reason: &'static str,
    pub last_inventory_change: &'static str,
    pub last_event_id: Option<event_log::EventId>,
    pub load_event_id: Option<event_log::EventId>,
    pub start_event_id: Option<event_log::EventId>,
    pub hot_swap_event_id: Option<event_log::EventId>,
    pub stop_event_id: Option<event_log::EventId>,
    pub drop_event_id: Option<event_log::EventId>,
}

#[derive(Clone, Copy)]
pub(crate) struct LifecycleCapture {
    pub before: Snapshot,
    pub after: Snapshot,
}

#[derive(Clone, Copy)]
pub(crate) struct State {
    pub(crate) service: ServiceState,
    pub(crate) state_migration: Option<HelloStateMigrationRecord>,
    pub(crate) hot_swap_probation: Option<HelloHotSwapProbationRecord>,
    pub(crate) applied_rollback: Option<AppliedRollbackRecord>,
    pub(crate) load_descriptor: LoadDescriptor,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            service: ServiceState::new(),
            state_migration: None,
            hot_swap_probation: None,
            applied_rollback: None,
            load_descriptor: LOAD_DESCRIPTOR,
        }
    }

    pub(crate) fn snapshot(self) -> Snapshot {
        let service = self.service;
        Snapshot {
            loaded: service.loaded,
            running: service.running,
            generation: service.generation,
            state_counter: service.state_counter,
            state_migration: self.state_migration,
            hot_swap_probation: self.hot_swap_probation,
            applied_rollback: self.applied_rollback,
            load_descriptor: self.load_descriptor,
            last_action: service.last_action,
            last_reason: service.last_reason,
            last_inventory_change: service.last_inventory_change,
            last_event_id: service.last_event_id,
            load_event_id: service.load_event_id,
            start_event_id: service.start_event_id,
            hot_swap_event_id: service.hot_swap_event_id,
            stop_event_id: service.stop_event_id,
            drop_event_id: service.drop_event_id,
        }
    }
}

impl core::ops::Deref for State {
    type Target = ServiceState;

    fn deref(&self) -> &Self::Target {
        &self.service
    }
}

impl core::ops::DerefMut for State {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.service
    }
}

pub(crate) static STATE: Mutex<State> = Mutex::new(State::new());
pub(crate) static RETAINED_RECOVERY_ROLLBACK_INSPECT_SOURCE: Mutex<
    Option<RecoveryRollbackInspectSourceReference>,
> = Mutex::new(None);

pub(crate) fn loaded_snapshot() -> Option<Snapshot> {
    let state = STATE.lock();
    if state.loaded {
        Some(state.snapshot())
    } else {
        None
    }
}

pub(crate) fn is_load_start_method(method: &str) -> bool {
    load_request(method).is_some()
}

pub(crate) fn is_stop_method(method: &str) -> bool {
    target_arg_matches(method, "service.stop")
}

pub(crate) fn is_start_method(method: &str) -> bool {
    target_arg_matches(method, "service.start")
}

pub(crate) fn is_restart_method(method: &str) -> bool {
    target_arg_matches(method, "service.restart")
}

pub(crate) fn is_hot_swap_method(method: &str) -> bool {
    hot_swap_request(method).is_some() || reset_state_hot_swap_target(method)
}

pub(crate) fn is_drop_method(method: &str) -> bool {
    target_arg_matches(method, "service.drop")
}

pub(crate) fn is_health_method(method: &str) -> bool {
    target_arg_matches(method, "service.health")
}

pub(crate) fn is_rollback_preview_method(method: &str) -> bool {
    target_arg_matches(method, "service.rollback_preview")
}

pub(crate) fn is_rollback_apply_method(method: &str) -> bool {
    target_arg_matches(method, "service.rollback_apply")
}

pub(crate) fn is_recovery_rollback_inspect_method(method: &str) -> bool {
    target_arg_matches(method, "recovery.rollback_inspect")
}

pub(crate) fn is_recovery_rollback_materialize_dry_run_method(method: &str) -> bool {
    target_arg_matches(method, "recovery.rollback_materialize_dry_run")
}

pub(crate) fn is_recovery_rollback_inspect_source_reference_selftest_method(method: &str) -> bool {
    method_eq(
        method,
        "recovery.rollback_inspect_source_reference_selftest",
    )
}

pub(crate) fn is_descriptor_source_trust_selftest_method(method: &str) -> bool {
    method_eq(method, "service.descriptor_source_trust_selftest")
}

pub(crate) fn is_artifact_reference_trust_selftest_method(method: &str) -> bool {
    method_eq(method, "service.artifact_reference_trust_selftest")
}

pub(crate) fn is_artifact_load_plan_preflight_selftest_method(method: &str) -> bool {
    method_eq(method, "service.artifact_load_plan_preflight_selftest")
}

pub(crate) fn emit_load_start(method: &str) -> &'static str {
    let Some(request) = load_request(method) else {
        return "module.load_ephemeral";
    };
    let capture = load_start(request.source_method, request.descriptor);
    emit_response(
        request.source_method,
        raios_core::hello_lifecycle_projection::LifecycleAction::Load,
        capture,
        request.descriptor,
    );
    request.source_method
}

pub(crate) fn emit_stop(_method: &str) -> &'static str {
    let capture = stop("service.stop");
    emit_response(
        "service.stop",
        raios_core::hello_lifecycle_projection::LifecycleAction::Stop,
        capture,
        capture.after.load_descriptor,
    );
    "service.stop"
}

pub(crate) fn emit_start(_method: &str) -> &'static str {
    let capture = start("service.start");
    emit_response(
        "service.start",
        raios_core::hello_lifecycle_projection::LifecycleAction::Start,
        capture,
        capture.after.load_descriptor,
    );
    "service.start"
}

pub(crate) fn emit_restart(_method: &str) -> &'static str {
    let capture = restart("service.restart");
    emit_response(
        "service.restart",
        raios_core::hello_lifecycle_projection::LifecycleAction::Restart,
        capture,
        capture.after.load_descriptor,
    );
    "service.restart"
}

pub(crate) fn emit_hot_swap(method: &str) -> &'static str {
    if let Some(request) = hot_swap_request(method) {
        let capture = hot_swap(request.source_method, request.descriptor);
        emit_response(
            request.source_method,
            raios_core::hello_lifecycle_projection::LifecycleAction::HotSwap,
            capture,
            request.descriptor,
        );
        request.source_method
    } else if reset_state_hot_swap_target(method) {
        let (snapshot, event_id, migration) = denied_reset_state_hot_swap();
        emit_hot_swap_state_migration_denied("service.hot_swap", snapshot, event_id, migration);
        "service.hot_swap"
    } else {
        "service.hot_swap"
    }
}

pub(crate) fn emit_drop(_method: &str) -> &'static str {
    let capture = drop_service("service.drop");
    emit_response(
        "service.drop",
        raios_core::hello_lifecycle_projection::LifecycleAction::Drop,
        capture,
        capture.after.load_descriptor,
    );
    "service.drop"
}

pub(crate) fn emit_health(_method: &str) -> &'static str {
    let (capture, event_id) = health_probe("service.health");
    emit_health_response("service.health", capture, event_id);
    "service.health"
}

pub(crate) fn emit_rollback_preview(_method: &str) -> &'static str {
    let (snapshot, event_id) = rollback_preview("service.rollback_preview");
    emit_rollback_preview_response("service.rollback_preview", snapshot, event_id);
    "service.rollback_preview"
}

pub(crate) fn emit_rollback_apply(_method: &str) -> &'static str {
    match rollback_apply("service.rollback_apply") {
        RollbackApplyResult::Denied { snapshot, event_id } => {
            emit_rollback_apply_denied("service.rollback_apply", snapshot, event_id);
        }
        RollbackApplyResult::Applied {
            pre_apply_snapshot,
            snapshot,
            event_id,
            proof,
        } => {
            emit_rollback_apply_applied(
                "service.rollback_apply",
                pre_apply_snapshot,
                snapshot,
                event_id,
                proof,
            );
        }
    }
    "service.rollback_apply"
}

pub(crate) fn emit_recovery_rollback_inspect(
    _method: &str,
    event_id: event_log::EventId,
) -> &'static str {
    let state = STATE.lock();
    let snapshot = state.snapshot();
    drop(state);
    emit_recovery_rollback_inspect_response("recovery.rollback_inspect", snapshot, event_id);
    "recovery.rollback_inspect"
}

pub(crate) fn emit_recovery_rollback_materialize_dry_run(
    _method: &str,
    event_id: event_log::EventId,
) -> &'static str {
    let state = STATE.lock();
    let snapshot = state.snapshot();
    drop(state);
    emit_recovery_rollback_materialize_dry_run_response(
        "recovery.rollback_materialize_dry_run",
        snapshot,
        event_id,
    );
    "recovery.rollback_materialize_dry_run"
}

pub(crate) fn emit_recovery_rollback_inspect_source_reference_selftest() -> &'static str {
    let method = "recovery.rollback_inspect_source_reference_selftest";
    let cases = event_log::hello_recovery_rollback_inspect_source_reference_selftest();
    let mut passed_count = 0usize;
    let mut idx = 0usize;
    while idx < cases.len() {
        if cases[idx].passed {
            passed_count += 1;
        }
        idx += 1;
    }

    let mut case_values = Vec::new();
    idx = 0;
    while idx < cases.len() {
        let case = cases[idx];
        case_values.push(V::InlineObject(vec![
            j!("name" => V::Str(case.name)),
            j!("expected_status" => V::Str(case.expected_status)),
            j!("expected_reason" => V::Str(case.expected_reason)),
            j!("actual_status" => V::Str(case.actual_status)),
            j!("actual_reason" => V::Str(case.actual_reason)),
            j!("source_event_retained" => V::Bool(case.source_event_retained)),
            j!("audit_event_retained" => V::Bool(case.audit_event_retained)),
            j!("validated" => V::Bool(case.validated)),
            j!("passed" => V::Bool(case.passed)),
        ]));
        idx += 1;
    }
    begin_response(method);
    emit_record_fields(
        vec![
            j!("schema" => V::Str(HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_SELFTEST_SCHEMA)),
            j!("id" => V::Str(HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_SELFTEST_ID)),
            j!("scope" => V::Str("current_boot")),
            j!("classification" => V::Str("local_only")),
            j!("persistence" => V::Str("none")),
            j!("test_infrastructure" => V::Bool(true)),
            j!("read_only" => V::Bool(true)),
            j!("mutates_global_event_log" => V::Bool(false)),
            j!("creates_source_reference_events" => V::Bool(false)),
            j!("diagnostic_hash" => V::Sha256(recovery_rollback_inspect_source_reference_selftest_hash())),
            j!("service_id" => V::Str(SERVICE_ID)),
            j!("validated_binding_schema" => V::Str("raios.recovery_rollback_inspect_source_reference_binding.v0")),
            j!("case_count" => V::U64(cases.len() as u64)),
            j!("passed_count" => V::U64(passed_count as u64)),
            j!("all_passed" => V::Bool(passed_count == cases.len())),
            j!("covered_rejections" => V::Array(vec![
                V::Str("missing_or_unretained_source_event_id"),
                V::Str("wrong_source_read_binding"),
                V::Str("missing_or_unretained_audit_event_id"),
                V::Str("wrong_audit_event_variant"),
                V::Str("substituted_hashes_denied"),
                V::Str("authorizing_source_reference_denied"),
            ])),
            j!("cases" => V::Array(case_values)),
            j!("denied_surfaces" => V::Object(vec![
                j!("rollback_apply" => V::Str("denied")),
                j!("durable_media_write" => V::Str("denied")),
                j!("durable_audit" => V::Str("denied")),
                j!("rollback_store_write" => V::Str("denied")),
                j!("transaction_append" => V::Str("denied")),
                j!("persistence" => V::Str("denied")),
                j!("external_artifact_bytes" => V::Str("denied")),
                j!("candidate_execution" => V::Str("denied")),
                j!("executable_mapping" => V::Str("denied")),
                j!("provider_auto_load" => V::Str("denied")),
                j!("broad_mutation" => V::Str("denied")),
                j!("installed_rollback_state" => V::Str("denied")),
            ])),
        ],
        6,
    );
    end_response(method);
    method
}

pub(crate) fn emit_descriptor_source_trust_selftest() -> &'static str {
    let method = "service.descriptor_source_trust_selftest";
    let cases = descriptor_sources::hello_descriptor_source_trust_selftest_cases();
    let mut passed_count = 0usize;
    let mut idx = 0usize;
    while idx < cases.len() {
        if cases[idx].passed {
            passed_count += 1;
        }
        idx += 1;
    }
    let mut case_values = Vec::new();
    idx = 0;
    while idx < cases.len() {
        let case = cases[idx];
        case_values.push(V::InlineObject(vec![
            j!("name" => V::Str(case.name)),
            j!("expected_accept" => V::Bool(case.expected_accept)),
            j!("actual_accept" => V::Bool(case.actual_accept)),
            j!("passed" => V::Bool(case.passed)),
            j!("reason" => V::Str(case.reason)),
        ]));
        idx += 1;
    }
    begin_response(method);
    emit_record_fields(
        vec![
            j!("schema" => V::Str("raios.descriptor_source_trust_selftest.v0")),
            j!("id" => V::Str(descriptor_sources::HELLO_DESCRIPTOR_SOURCE_TRUST_SELFTEST_ID)),
            j!("scope" => V::Str("current_boot")),
            j!("classification" => V::Str("local_only")),
            j!("persistence" => V::Str("none")),
            j!("read_only" => V::Bool(true)),
            j!("diagnostic_hash" => V::Sha256(descriptor_sources::hello_descriptor_source_trust_selftest_hash())),
            j!("service_id" => V::Str(SERVICE_ID)),
            j!("descriptor_source_locator" => V::Str(LOAD_DESCRIPTOR_SOURCE_LOCATOR)),
            j!("descriptor_source_kind" => V::Str(LOAD_DESCRIPTOR_SOURCE_KIND)),
            j!("signature_envelope" => descriptor_source_signature_envelope_value(LOAD_DESCRIPTOR)),
            j!("case_count" => V::U64(cases.len() as u64)),
            j!("passed_count" => V::U64(passed_count as u64)),
            j!("all_passed" => V::Bool(passed_count == cases.len())),
            j!("cases" => V::Array(case_values)),
            j!("denied_surfaces" => V::Object(vec![
                j!("descriptor_bytes_intake" => V::Str("denied")),
                j!("external_artifact_load" => V::Str("denied")),
                j!("persistent_install" => V::Str("denied")),
                j!("durable_audit" => V::Str("denied")),
                j!("rollback_install" => V::Str("denied")),
                j!("broad_mutation" => V::Str("denied")),
            ])),
        ],
        6,
    );
    end_response(method);
    method
}

pub(crate) fn emit_artifact_reference_trust_selftest() -> &'static str {
    let method = "service.artifact_reference_trust_selftest";
    let cases = descriptor_sources::hello_artifact_reference_trust_selftest_cases();
    let mut passed_count = 0usize;
    let mut idx = 0usize;
    while idx < cases.len() {
        if cases[idx].passed {
            passed_count += 1;
        }
        idx += 1;
    }
    let mut case_values = Vec::new();
    idx = 0;
    while idx < cases.len() {
        let case = cases[idx];
        case_values.push(V::InlineObject(vec![
            j!("name" => V::Str(case.name)),
            j!("expected_accept" => V::Bool(case.expected_accept)),
            j!("actual_accept" => V::Bool(case.actual_accept)),
            j!("passed" => V::Bool(case.passed)),
            j!("reason" => V::Str(case.reason)),
        ]));
        idx += 1;
    }
    begin_response(method);
    emit_record_fields(
        vec![
            j!("schema" => V::Str("raios.builtin_artifact_reference_trust_selftest.v0")),
            j!("id" => V::Str(descriptor_sources::HELLO_ARTIFACT_REFERENCE_TRUST_SELFTEST_ID)),
            j!("scope" => V::Str("current_boot")),
            j!("classification" => V::Str("local_only")),
            j!("persistence" => V::Str("none")),
            j!("read_only" => V::Bool(true)),
            j!("mutates_global_event_log" => V::Bool(false)),
            j!("diagnostic_hash" => V::Sha256(descriptor_sources::hello_artifact_reference_trust_selftest_hash())),
            j!("service_id" => V::Str(SERVICE_ID)),
            j!("artifact_id" => V::Str(ARTIFACT_ID)),
            j!("artifact_reference" => artifact_reference_value(LOAD_DESCRIPTOR)),
            j!("identity_signature_envelope" => artifact_identity_signature_envelope_value(LOAD_DESCRIPTOR)),
            j!("case_count" => V::U64(cases.len() as u64)),
            j!("passed_count" => V::U64(passed_count as u64)),
            j!("all_passed" => V::Bool(passed_count == cases.len())),
            j!("cases" => V::Array(case_values)),
            j!("denied_surfaces" => V::Object(vec![
                j!("artifact_bytes_intake" => V::Str("denied")),
                j!("artifact_load" => V::Str("denied")),
                j!("executable_mapping" => V::Str("denied")),
                j!("persistent_install" => V::Str("denied")),
                j!("durable_audit" => V::Str("denied")),
                j!("rollback_install" => V::Str("denied")),
                j!("broad_mutation" => V::Str("denied")),
            ])),
        ],
        6,
    );
    end_response(method);
    method
}

pub(crate) fn emit_artifact_load_plan_preflight_selftest() -> &'static str {
    let method = "service.artifact_load_plan_preflight_selftest";
    let cases = artifact_load_plan_preflight_selftest_cases();
    let mut passed_count = 0usize;
    let mut idx = 0usize;
    while idx < cases.len() {
        if cases[idx].passed {
            passed_count += 1;
        }
        idx += 1;
    }
    let mut case_values = Vec::new();
    idx = 0;
    while idx < cases.len() {
        let case = cases[idx];
        case_values.push(V::InlineObject(vec![
            j!("name" => V::Str(case.name)),
            j!("expected_accept" => V::Bool(case.expected_accept)),
            j!("actual_accept" => V::Bool(case.actual_accept)),
            j!("passed" => V::Bool(case.passed)),
            j!("reason" => V::Str(case.reason)),
        ]));
        idx += 1;
    }
    begin_response(method);
    emit_record_fields(
        vec![
            j!("schema" => V::Str(ARTIFACT_LOAD_PLAN_PREFLIGHT_SELFTEST_SCHEMA)),
            j!("id" => V::Str(ARTIFACT_LOAD_PLAN_PREFLIGHT_SELFTEST_ID)),
            j!("scope" => V::Str("current_boot")),
            j!("classification" => V::Str("local_only")),
            j!("persistence" => V::Str("none")),
            j!("read_only" => V::Bool(true)),
            j!("mutates_global_event_log" => V::Bool(false)),
            j!("diagnostic_hash" => V::Sha256(artifact_load_plan_preflight_selftest_hash())),
            j!("service_id" => V::Str(SERVICE_ID)),
            j!("artifact_id" => V::Str(ARTIFACT_ID)),
            j!("service_slot_intent_id" => V::Str(SERVICE_SLOT_INTENT_ID)),
            j!("ram_only_service_slot_id" => V::Str(RAM_ONLY_SERVICE_SLOT_ID)),
            j!("artifact_load_plan_preflight" => artifact_load_plan_preflight_value(LOAD_DESCRIPTOR)),
            j!("case_count" => V::U64(cases.len() as u64)),
            j!("passed_count" => V::U64(passed_count as u64)),
            j!("all_passed" => V::Bool(passed_count == cases.len())),
            j!("cases" => V::Array(case_values)),
            j!("denied_surfaces" => V::Object(vec![
                j!("external_artifact_bytes" => V::Str("denied")),
                j!("candidate_artifact_execution" => V::Str("denied")),
                j!("artifact_load" => V::Str("denied")),
                j!("executable_mapping" => V::Str("denied")),
                j!("persistent_install" => V::Str("denied")),
                j!("durable_audit" => V::Str("denied")),
                j!("rollback_install" => V::Str("denied")),
                j!("broad_mutation" => V::Str("denied")),
            ])),
        ],
        6,
    );
    end_response(method);
    method
}
