use super::*;
use crate::current_boot_service::ServiceState;

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

    begin_response(method);
    raw("      \"schema\": ");
    json_str(HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_SELFTEST_SCHEMA);
    raw_line(",");
    raw("      \"id\": ");
    json_str(HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_SELFTEST_ID);
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"read_only\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_source_reference_events\": false,");
    raw("      \"diagnostic_hash\": ");
    json_sha256(recovery_rollback_inspect_source_reference_selftest_hash());
    raw_line(",");
    raw("      \"service_id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw_line("      \"validated_binding_schema\": \"raios.recovery_rollback_inspect_source_reference_binding.v0\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed_count\": ");
    raw_fmt(format_args!("{}", passed_count));
    raw_line(",");
    raw("      \"all_passed\": ");
    raw_bool(passed_count == cases.len());
    raw_line(",");
    raw_line("      \"covered_rejections\": [");
    raw_line("        \"missing_or_unretained_source_event_id\",");
    raw_line("        \"wrong_source_read_binding\",");
    raw_line("        \"missing_or_unretained_audit_event_id\",");
    raw_line("        \"wrong_audit_event_variant\",");
    raw_line("        \"substituted_hashes_denied\",");
    raw_line("        \"authorizing_source_reference_denied\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        let case = cases[idx];
        raw("        {\"name\": ");
        json_str(case.name);
        raw(", \"expected_status\": ");
        json_str(case.expected_status);
        raw(", \"expected_reason\": ");
        json_str(case.expected_reason);
        raw(", \"actual_status\": ");
        json_str(case.actual_status);
        raw(", \"actual_reason\": ");
        json_str(case.actual_reason);
        raw(", \"source_event_retained\": ");
        raw_bool(case.source_event_retained);
        raw(", \"audit_event_retained\": ");
        raw_bool(case.audit_event_retained);
        raw(", \"validated\": ");
        raw_bool(case.validated);
        raw(", \"passed\": ");
        raw_bool(case.passed);
        raw("}");
        if idx + 1 != cases.len() {
            raw(",");
        }
        raw_line("");
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"denied_surfaces\": {");
    raw_line("        \"rollback_apply\": \"denied\",");
    raw_line("        \"durable_media_write\": \"denied\",");
    raw_line("        \"durable_audit\": \"denied\",");
    raw_line("        \"rollback_store_write\": \"denied\",");
    raw_line("        \"transaction_append\": \"denied\",");
    raw_line("        \"persistence\": \"denied\",");
    raw_line("        \"external_artifact_bytes\": \"denied\",");
    raw_line("        \"candidate_execution\": \"denied\",");
    raw_line("        \"executable_mapping\": \"denied\",");
    raw_line("        \"provider_auto_load\": \"denied\",");
    raw_line("        \"broad_mutation\": \"denied\",");
    raw_line("        \"installed_rollback_state\": \"denied\"");
    raw_line("      }");
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
    begin_response(method);
    raw_line("      \"schema\": \"raios.descriptor_source_trust_selftest.v0\",");
    raw("      \"id\": ");
    json_str(descriptor_sources::HELLO_DESCRIPTOR_SOURCE_TRUST_SELFTEST_ID);
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"read_only\": true,");
    raw("      \"diagnostic_hash\": ");
    json_sha256(descriptor_sources::hello_descriptor_source_trust_selftest_hash());
    raw_line(",");
    raw("      \"service_id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw("      \"descriptor_source_locator\": ");
    json_str(LOAD_DESCRIPTOR_SOURCE_LOCATOR);
    raw_line(",");
    raw("      \"descriptor_source_kind\": ");
    json_str(LOAD_DESCRIPTOR_SOURCE_KIND);
    raw_line(",");
    raw("      \"signature_envelope\": ");
    emit_descriptor_source_signature_envelope(LOAD_DESCRIPTOR);
    raw_line(",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed_count\": ");
    raw_fmt(format_args!("{}", passed_count));
    raw_line(",");
    raw("      \"all_passed\": ");
    raw_bool(passed_count == cases.len());
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        let case = cases[idx];
        raw("        {\"name\": ");
        json_str(case.name);
        raw(", \"expected_accept\": ");
        raw_bool(case.expected_accept);
        raw(", \"actual_accept\": ");
        raw_bool(case.actual_accept);
        raw(", \"passed\": ");
        raw_bool(case.passed);
        raw(", \"reason\": ");
        json_str(case.reason);
        raw("}");
        if idx + 1 != cases.len() {
            raw(",");
        }
        raw_line("");
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"denied_surfaces\": {");
    raw_line("        \"descriptor_bytes_intake\": \"denied\",");
    raw_line("        \"external_artifact_load\": \"denied\",");
    raw_line("        \"persistent_install\": \"denied\",");
    raw_line("        \"durable_audit\": \"denied\",");
    raw_line("        \"rollback_install\": \"denied\",");
    raw_line("        \"broad_mutation\": \"denied\"");
    raw_line("      }");
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
    begin_response(method);
    raw_line("      \"schema\": \"raios.builtin_artifact_reference_trust_selftest.v0\",");
    raw("      \"id\": ");
    json_str(descriptor_sources::HELLO_ARTIFACT_REFERENCE_TRUST_SELFTEST_ID);
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"read_only\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw("      \"diagnostic_hash\": ");
    json_sha256(descriptor_sources::hello_artifact_reference_trust_selftest_hash());
    raw_line(",");
    raw("      \"service_id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw("      \"artifact_id\": ");
    json_str(ARTIFACT_ID);
    raw_line(",");
    raw("      \"artifact_reference\": ");
    emit_artifact_reference(LOAD_DESCRIPTOR);
    raw_line(",");
    raw("      \"identity_signature_envelope\": ");
    emit_artifact_identity_signature_envelope(LOAD_DESCRIPTOR);
    raw_line(",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed_count\": ");
    raw_fmt(format_args!("{}", passed_count));
    raw_line(",");
    raw("      \"all_passed\": ");
    raw_bool(passed_count == cases.len());
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        let case = cases[idx];
        raw("        {\"name\": ");
        json_str(case.name);
        raw(", \"expected_accept\": ");
        raw_bool(case.expected_accept);
        raw(", \"actual_accept\": ");
        raw_bool(case.actual_accept);
        raw(", \"passed\": ");
        raw_bool(case.passed);
        raw(", \"reason\": ");
        json_str(case.reason);
        raw("}");
        if idx + 1 != cases.len() {
            raw(",");
        }
        raw_line("");
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"denied_surfaces\": {");
    raw_line("        \"artifact_bytes_intake\": \"denied\",");
    raw_line("        \"artifact_load\": \"denied\",");
    raw_line("        \"executable_mapping\": \"denied\",");
    raw_line("        \"persistent_install\": \"denied\",");
    raw_line("        \"durable_audit\": \"denied\",");
    raw_line("        \"rollback_install\": \"denied\",");
    raw_line("        \"broad_mutation\": \"denied\"");
    raw_line("      }");
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
    begin_response(method);
    raw("      \"schema\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_SELFTEST_SCHEMA);
    raw_line(",");
    raw("      \"id\": ");
    json_str(ARTIFACT_LOAD_PLAN_PREFLIGHT_SELFTEST_ID);
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"read_only\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw("      \"diagnostic_hash\": ");
    json_sha256(artifact_load_plan_preflight_selftest_hash());
    raw_line(",");
    raw("      \"service_id\": ");
    json_str(SERVICE_ID);
    raw_line(",");
    raw("      \"artifact_id\": ");
    json_str(ARTIFACT_ID);
    raw_line(",");
    raw("      \"service_slot_intent_id\": ");
    json_str(SERVICE_SLOT_INTENT_ID);
    raw_line(",");
    raw("      \"ram_only_service_slot_id\": ");
    json_str(RAM_ONLY_SERVICE_SLOT_ID);
    raw_line(",");
    raw("      \"artifact_load_plan_preflight\": ");
    emit_artifact_load_plan_preflight(LOAD_DESCRIPTOR);
    raw_line(",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed_count\": ");
    raw_fmt(format_args!("{}", passed_count));
    raw_line(",");
    raw("      \"all_passed\": ");
    raw_bool(passed_count == cases.len());
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        let case = cases[idx];
        raw("        {\"name\": ");
        json_str(case.name);
        raw(", \"expected_accept\": ");
        raw_bool(case.expected_accept);
        raw(", \"actual_accept\": ");
        raw_bool(case.actual_accept);
        raw(", \"passed\": ");
        raw_bool(case.passed);
        raw(", \"reason\": ");
        json_str(case.reason);
        raw("}");
        if idx + 1 != cases.len() {
            raw(",");
        }
        raw_line("");
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"denied_surfaces\": {");
    raw_line("        \"external_artifact_bytes\": \"denied\",");
    raw_line("        \"candidate_artifact_execution\": \"denied\",");
    raw_line("        \"artifact_load\": \"denied\",");
    raw_line("        \"executable_mapping\": \"denied\",");
    raw_line("        \"persistent_install\": \"denied\",");
    raw_line("        \"durable_audit\": \"denied\",");
    raw_line("        \"rollback_install\": \"denied\",");
    raw_line("        \"broad_mutation\": \"denied\"");
    raw_line("      }");
    end_response(method);
    method
}
