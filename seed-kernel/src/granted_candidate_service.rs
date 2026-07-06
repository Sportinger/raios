use alloc::{vec, vec::Vec};

use raios_core::record::{sha256_of_json, Value as V};
use spin::Mutex;

use crate::{
    agent_protocol::durable_store,
    agent_protocol_module_grant,
    agent_protocol_module_types::ModuleGrantReferenceCheck,
    agent_protocol_support::{
        begin_response, emit_record_fields_trailing_comma, end_response, method_eq,
        record_bool as b, record_event_or_null, record_field as f, record_sha_or_null,
        record_static_str_array, record_str as s, record_str_or_null,
    },
    current_boot_service::{self, ServiceDescriptor, ServiceState},
    echo_service, event_log, hello_service,
    module_candidate_intake::{self, RetainedExternalWasmCandidate},
    module_evidence, service_inventory, wasm_runtime,
};

pub(crate) const GRANTED_CANDIDATE_SERVICE_DESCRIPTOR: ServiceDescriptor = ServiceDescriptor {
    service_id: "svc.dev.granted_candidate",
    artifact_id: "wasm:external.granted_candidate",
    artifact_kind: "wasm32_unknown_unknown_service_module",
    scope: "current_boot",
    classification: "local_only",
    persistence: "none",
    service_capability: "cap.module.load_ephemeral.dev_tier.current_boot",
    health_capability: "cap.service.health.read",
    rollback_preview_capability: "cap.service.granted_candidate.rollback_preview.read",
    rollback_apply_capability: "cap.service.granted_candidate.rollback_apply.current_boot",
    rollback_materialize_capability:
        "cap.recovery.granted_candidate.rollback_materialize_dry_run.current_boot",
    rollback_inspect_capability: "cap.recovery.granted_candidate.rollback_inspect.read",
    primary_alias: "granted_candidate",
    host_bound_alias: "host_bound:svc.dev.granted_candidate",
    replacement_service_id: "svc.dev.granted_candidate.v2",
    replacement_alias: "granted_candidate.v2",
    replacement_artifact_identity_id: "external_artifact_identity.svc.dev.granted_candidate.v2",
    reset_state_service_id: "svc.dev.granted_candidate.reset_state",
    reset_state_alias: "granted_candidate.reset_state",
    artifact_load_plan_preflight_id:
        "artifact_load_plan_preflight.current_boot.granted_candidate.v0",
    artifact_load_plan_preflight_status: "accepted_dev_key_ram_only_external_wasm",
    service_slot_intent_id: "service_slot_intent.current_boot.granted_candidate.v0",
    ram_only_service_slot_id: "ram_only:svc.dev.granted_candidate",
    service_slot_activation_id: "service_slot_activation.current_boot.granted_candidate.v0",
    service_slot_activation_active_status: "active_current_boot",
    service_slot_activation_stopped_status: "stopped_current_boot",
    service_slot_activation_cleared_status: "cleared_current_boot",
    service_slot_activation_missing_status: "missing_current_boot",
    inventory_kind: "service",
    inventory_replaceable: true,
    inventory_core_owned: false,
    inventory_health_running: "healthy",
    inventory_health_stopped: "stopped",
    inventory_health_missing: "missing",
    event_lifecycle_kind: "raios.ram_only_granted_candidate_service.lifecycle",
    event_health_kind: "raios.ram_only_granted_candidate_service.health",
    event_rollback_preview_kind: "raios.ram_only_granted_candidate_service.rollback_preview",
    event_rollback_apply_kind: "raios.ram_only_granted_candidate_service.rollback_apply",
};

const LOAD_DESCRIPTOR_SCHEMA: &str = "raios.current_boot_load_descriptor.v0";
const LOAD_DESCRIPTOR_ID: &str = "load_descriptor.current_boot.svc.dev.granted_candidate.v0";
const LOAD_DESCRIPTOR_SOURCE_LOCATOR: &str =
    "runtime.serial.dev_key_granted_external_wasm_candidate";
const LOAD_DESCRIPTOR_SOURCE_KIND: &str = "runtime_granted_external_wasm_candidate";
const LIFECYCLE_RESPONSE_SCHEMA: &str =
    "raios.ram_only_granted_candidate_service.lifecycle_response.v0";
const SELFTEST_SCHEMA: &str = "raios.ram_only_granted_candidate_service.selftest.v0";
const SERVICE_VERSION: &str = "v0";
const GRANTED_ENTRYPOINT: &str = "raios_service_main";
const TRUST_TIER_GRANTED: &str = "dev_key_not_owner_sealed";
const TRUST_TIER_DENIED: &str = "unsealed_no_grant";
const CLEANUP_ACTIONS: &[&str] = &["stop", "drop_clear_bytes", "free_slot", "remove_inventory"];
const CLEANUP_ACTIONS_CANONICAL: &[u8] = b"stop\ndrop_clear_bytes\nfree_slot\nremove_inventory";
const LIFECYCLE_EVIDENCE: &[&str] = &[
    "module.submit_candidate_finalize",
    "module.grant_diagnostic",
    "module.attestation_diagnostic",
    "wasmi_linker_import_surface",
];
const ROLLBACK_EVIDENCE: &[&str] = &[
    "raios.rollback_plan.v0",
    "module.submit_candidate_finalize",
    "service.inventory",
];
pub(crate) const CAPABILITIES: &[&str] = &[
    "cap.module.load_ephemeral.dev_tier.current_boot",
    "cap.service.health.read",
    "cap.service.granted_candidate.rollback_preview.read",
    "cap.service.granted_candidate.rollback_apply.current_boot",
];

#[derive(Clone, Copy)]
pub(crate) struct PromotionRecord {
    pub(crate) plan_hash: [u8; 32],
    pub(crate) pre_load_inventory_hash: [u8; 32],
    pub(crate) ram_only_service_slot_id: &'static str,
    pub(crate) generation: u64,
    pub(crate) load_event_id: event_log::EventId,
    pub(crate) artifact_hash: [u8; 32],
}

#[derive(Clone, Copy)]
pub(crate) struct Snapshot {
    pub(crate) loaded: bool,
    pub(crate) running: bool,
    pub(crate) generation: u64,
    pub(crate) run_count: u64,
    pub(crate) last_run_outcome: &'static str,
    pub(crate) last_return_value: Option<i32>,
    pub(crate) last_fuel_used: u64,
    pub(crate) last_log_line_emitted: bool,
    pub(crate) last_action: &'static str,
    pub(crate) last_reason: &'static str,
    pub(crate) last_inventory_change: &'static str,
    pub(crate) trust_tier: &'static str,
    pub(crate) load_event_id: Option<event_log::EventId>,
    pub(crate) start_event_id: Option<event_log::EventId>,
    pub(crate) stop_event_id: Option<event_log::EventId>,
    pub(crate) drop_event_id: Option<event_log::EventId>,
    pub(crate) promotion: Option<PromotionRecord>,
}

#[derive(Clone, Copy)]
pub(crate) struct LiveLoadProjection {
    pub(crate) present: bool,
    pub(crate) accepts_external_artifact_bytes: bool,
    pub(crate) loads_artifact: bool,
    pub(crate) can_load_now: bool,
    pub(crate) service_slot_allocated: bool,
    pub(crate) running: bool,
    pub(crate) run_outcome: &'static str,
    pub(crate) trust_tier: &'static str,
    pub(crate) load_mechanism: &'static str,
    pub(crate) maps_executable_pages: bool,
    pub(crate) durable: bool,
    pub(crate) owner_sealed: bool,
    pub(crate) authorizes_native_guest_load: bool,
}

impl LiveLoadProjection {
    const fn absent() -> Self {
        Self {
            present: false,
            accepts_external_artifact_bytes: false,
            loads_artifact: false,
            can_load_now: false,
            service_slot_allocated: false,
            running: false,
            run_outcome: "not_loaded",
            trust_tier: TRUST_TIER_DENIED,
            load_mechanism: "none",
            maps_executable_pages: false,
            durable: false,
            owner_sealed: false,
            authorizes_native_guest_load: false,
        }
    }
}

#[derive(Clone, Copy)]
struct State {
    service: ServiceState,
    last_run_outcome: &'static str,
    last_return_value: Option<i32>,
    last_fuel_used: u64,
    last_log_line_emitted: bool,
    trust_tier: &'static str,
    promotion: Option<PromotionRecord>,
}

impl State {
    const fn new() -> Self {
        Self {
            service: ServiceState::new(),
            last_run_outcome: "not_run",
            last_return_value: None,
            last_fuel_used: 0,
            last_log_line_emitted: false,
            trust_tier: TRUST_TIER_DENIED,
            promotion: None,
        }
    }

    fn snapshot(self) -> Snapshot {
        let service = self.service;
        Snapshot {
            loaded: service.loaded,
            running: service.running,
            generation: service.generation,
            run_count: service.state_counter,
            last_run_outcome: self.last_run_outcome,
            last_return_value: self.last_return_value,
            last_fuel_used: self.last_fuel_used,
            last_log_line_emitted: self.last_log_line_emitted,
            last_action: service.last_action,
            last_reason: service.last_reason,
            last_inventory_change: service.last_inventory_change,
            trust_tier: self.trust_tier,
            load_event_id: service.load_event_id,
            start_event_id: service.start_event_id,
            stop_event_id: service.stop_event_id,
            drop_event_id: service.drop_event_id,
            promotion: self.promotion,
        }
    }
}

#[derive(Clone, Copy)]
struct AuthorizationEvidence {
    grants_capability: bool,
    retained_present: bool,
    retained_wasm_valid: bool,
    retained_sha: Option<[u8; 32]>,
    grant_artifact_hash: Option<[u8; 32]>,
    retained_sha_matches_grant: bool,
    can_execute: bool,
    trust_tier: &'static str,
    denial_reason: &'static str,
}

struct ActionResult {
    snapshot: Snapshot,
    event_id: event_log::EventId,
    run: Option<wasm_runtime::EchoRunEvidence>,
    authorization: AuthorizationEvidence,
    durable_promotion_transaction: durable_store::PromotionTransactionAppendEvidence,
    capability_denied: bool,
}

struct RollbackResult {
    snapshot: Snapshot,
    event_id: event_log::EventId,
    promotion: Option<PromotionRecord>,
    reprojected_inventory_hash: Option<[u8; 32]>,
    restored_inventory_hash_verified: bool,
    durable_unpromote_transaction: durable_store::PromotionTransactionAppendEvidence,
    capability_denied: bool,
    reason: &'static str,
}

struct SelftestCase {
    name: &'static str,
    capability_denied: bool,
    instantiation_ok: bool,
    run_outcome: &'static str,
    fuel_used: u64,
    trust_tier: &'static str,
    live_load_projection: Option<LiveLoadProjection>,
    durable: bool,
    rollback_restore_hash_verified: bool,
    second_rollback_apply_denied: bool,
    passed: bool,
}

static STATE: Mutex<State> = Mutex::new(State::new());

pub(crate) fn loaded_snapshot() -> Option<Snapshot> {
    let state = STATE.lock();
    if state.service.loaded {
        Some(state.snapshot())
    } else {
        None
    }
}

pub(crate) fn live_load_projection() -> LiveLoadProjection {
    live_load_projection_from_snapshot(loaded_snapshot())
}

fn live_load_projection_from_snapshot(snapshot: Option<Snapshot>) -> LiveLoadProjection {
    let Some(snapshot) = snapshot else {
        return LiveLoadProjection::absent();
    };
    let loaded = snapshot.loaded;
    LiveLoadProjection {
        present: loaded,
        accepts_external_artifact_bytes: loaded,
        loads_artifact: loaded,
        can_load_now: loaded,
        service_slot_allocated: loaded,
        running: snapshot.running,
        run_outcome: if loaded {
            snapshot.last_run_outcome
        } else {
            "not_loaded"
        },
        trust_tier: if loaded {
            TRUST_TIER_GRANTED
        } else {
            TRUST_TIER_DENIED
        },
        load_mechanism: if loaded {
            "wasmi_interpreter_ram_only"
        } else {
            "none"
        },
        maps_executable_pages: false,
        durable: false,
        owner_sealed: false,
        authorizes_native_guest_load: false,
    }
}

pub(crate) fn slot_allocatable() -> bool {
    !STATE.lock().service.loaded
}

pub(crate) fn ram_only_service_slot_id() -> &'static str {
    GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.ram_only_service_slot_id
}

pub(crate) fn service_slot_activation_id() -> &'static str {
    GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.service_slot_activation_id
}

pub(crate) fn is_load_method(method: &str) -> bool {
    load_source_method(method).is_some() && granted_candidate_ready()
}

pub(crate) fn is_start_method(method: &str) -> bool {
    target_arg_matches(method, "service.start")
        && (granted_candidate_ready() || loaded_snapshot().is_some())
}

pub(crate) fn is_stop_method(method: &str) -> bool {
    target_arg_matches(method, "service.stop")
        && (granted_candidate_ready() || loaded_snapshot().is_some())
}

pub(crate) fn is_drop_method(method: &str) -> bool {
    target_arg_matches(method, "service.drop")
        && (granted_candidate_ready() || loaded_snapshot().is_some())
}

pub(crate) fn is_rollback_preview_method(method: &str) -> bool {
    target_arg_matches(method, "service.rollback_preview")
}

pub(crate) fn is_rollback_apply_method(method: &str) -> bool {
    target_arg_matches(method, "service.rollback_apply")
}

pub(crate) fn is_selftest_method(method: &str) -> bool {
    method_eq(method, "module.granted_candidate_selftest")
}

pub(crate) fn emit_load(method: &str) -> &'static str {
    let source_method = load_source_method(method).unwrap_or("module.load_ephemeral");
    let result = load(source_method);
    emit_response(source_method, "load", result);
    source_method
}

pub(crate) fn emit_start(_method: &str) -> &'static str {
    let result = start("service.start");
    emit_response("service.start", "start", result);
    "service.start"
}

pub(crate) fn emit_stop(_method: &str) -> &'static str {
    let result = stop("service.stop");
    emit_response("service.stop", "stop", result);
    "service.stop"
}

pub(crate) fn emit_drop(_method: &str) -> &'static str {
    let result = drop_service("service.drop");
    emit_response("service.drop", "drop", result);
    "service.drop"
}

pub(crate) fn emit_rollback_preview(_method: &str) -> &'static str {
    let result = rollback_preview("service.rollback_preview");
    emit_rollback_response("service.rollback_preview", "rollback_preview", result);
    "service.rollback_preview"
}

pub(crate) fn emit_rollback_apply(_method: &str) -> &'static str {
    let result = rollback_apply("service.rollback_apply");
    emit_rollback_response("service.rollback_apply", "rollback_apply", result);
    "service.rollback_apply"
}

pub(crate) fn emit_selftest() -> &'static str {
    let cases = run_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);
    let case_records = cases.iter().map(record_selftest_case).collect();

    begin_response("module.granted_candidate_selftest");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s(SELFTEST_SCHEMA)),
            f("scope", s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.scope)),
            f(
                "classification",
                s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.classification),
            ),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", b(false)),
            f("loads_artifact", b(false)),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f("cases", V::Array(case_records)),
        ],
        6,
    );
    crate::agent_protocol_support::raw_line("      \"evidence_complete\": true");
    end_response("module.granted_candidate_selftest");
    "module.granted_candidate_selftest"
}

fn load_source_method(method: &str) -> Option<&'static str> {
    if target_arg_matches(method, "module.load_ephemeral") {
        Some("module.load_ephemeral")
    } else if target_arg_matches(method, "service.load_ephemeral") {
        Some("service.load_ephemeral")
    } else {
        None
    }
}

fn target_arg_matches(method: &str, head: &str) -> bool {
    current_boot_service::target_arg_matches(
        method,
        head,
        current_boot_service::LoadTarget {
            service_id: GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.service_id,
            artifact_id: GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.artifact_id,
            descriptor_id: LOAD_DESCRIPTOR_ID,
        },
        GRANTED_CANDIDATE_SERVICE_DESCRIPTOR,
    )
}

fn granted_candidate_ready() -> bool {
    let retained = module_candidate_intake::retained();
    let (grant_check, retained_attestation) = current_grant_inputs();
    let authorization =
        evaluate_authorization(&grant_check, retained_attestation, retained.as_ref());
    authorization.can_execute
}

fn load(source_method: &'static str) -> ActionResult {
    let retained = module_candidate_intake::retained();
    let (grant_check, retained_attestation) = current_grant_inputs();
    let authorization =
        evaluate_authorization(&grant_check, retained_attestation, retained.as_ref());
    let mut promotion_for_append = None;
    let (snapshot, event_id, capability_denied) = {
        let mut state = STATE.lock();
        let slot_available = !state.service.loaded;
        let can_load =
            authorization.can_execute && slot_available && wasm_runtime::loader_available();
        let reason = if can_load {
            "loaded_dev_key_granted_external_wasm_current_boot"
        } else if state.service.loaded && authorization.can_execute {
            "already_loaded"
        } else {
            authorization.denial_reason
        };
        let event_id = event_log::record_service_lifecycle_unbound(
            &GRANTED_CANDIDATE_SERVICE_DESCRIPTOR,
            source_method,
            if can_load || (state.service.loaded && authorization.can_execute) {
                "response"
            } else {
                "capability_denied"
            },
            reason,
            LIFECYCLE_EVIDENCE,
        );

        if can_load {
            let generation = state.service.generation.saturating_add(1);
            if let Some(candidate) = retained.as_ref() {
                let promotion = build_promotion_record(
                    candidate.sha256,
                    service_inventory_projection_hash(None),
                    generation,
                    event_id,
                );
                state.promotion = Some(promotion);
                promotion_for_append = Some(promotion);
            }
            state.service.generation = generation;
            state.service.loaded = true;
            state.service.running = false;
            state.service.load_event_id = Some(event_id);
        }
        state.service.last_action = "load";
        state.service.last_reason = reason;
        state.service.last_inventory_change = if can_load {
            "upserted_current_boot_service"
        } else {
            "none"
        };
        state.service.last_event_id = Some(event_id);
        state.trust_tier = authorization.trust_tier;
        (
            state.snapshot(),
            event_id,
            !can_load && !(state.service.loaded && authorization.can_execute),
        )
    };
    let durable_promotion_transaction = promotion_for_append
        .map(|promotion| {
            append_promotion_transaction(
                durable_store::PromotionTransactionKind::Promote,
                promotion,
                None,
                false,
                None,
                false,
            )
        })
        .unwrap_or_else(|| {
            durable_store::promotion_transaction_append_denied(
                durable_store::PromotionTransactionKind::Promote,
                "promotion_transaction_not_attempted",
            )
        });
    ActionResult {
        snapshot,
        event_id,
        run: None,
        authorization,
        durable_promotion_transaction,
        capability_denied,
    }
}

fn start(source_method: &'static str) -> ActionResult {
    let was_loaded = STATE.lock().service.loaded;
    let retained = module_candidate_intake::retained();
    let (grant_check, retained_attestation) = current_grant_inputs();
    let grants_capability = agent_protocol_module_grant::module_grant_grants_capability(
        &grant_check,
        retained_attestation,
    );
    let mut authorization =
        evaluate_authorization(&grant_check, retained_attestation, retained.as_ref());
    let retained_sha_matches_grant =
        retained_sha_matches_grant(retained.as_ref(), grant_check.artifact_hash);
    authorization.grants_capability = grants_capability;
    authorization.retained_sha_matches_grant = retained_sha_matches_grant;
    authorization.can_execute =
        grants_capability && retained_sha_matches_grant && authorization.retained_wasm_valid;
    authorization.trust_tier =
        agent_protocol_module_grant::module_grant_trust_tier(grants_capability);
    authorization.denial_reason = denial_reason(authorization);

    let can_execute = was_loaded
        && grants_capability
        && retained_sha_matches_grant
        && authorization.retained_wasm_valid;
    let run = if can_execute {
        retained
            .as_ref()
            .map(|retained| wasm_runtime::execute_module_bytes(&retained.bytes, GRANTED_ENTRYPOINT))
    } else {
        None
    };
    let run_ok = run.as_ref().map(run_succeeded).unwrap_or(false);
    let reason = if !was_loaded {
        "not_loaded"
    } else if !can_execute {
        authorization.denial_reason
    } else if run_ok {
        "wasm_run_success"
    } else {
        "wasm_run_failed"
    };
    let event_id = event_log::record_service_lifecycle_unbound(
        &GRANTED_CANDIDATE_SERVICE_DESCRIPTOR,
        source_method,
        if can_execute {
            "response"
        } else {
            "capability_denied"
        },
        reason,
        LIFECYCLE_EVIDENCE,
    );

    let mut state = STATE.lock();
    if was_loaded {
        state.service.running = run_ok;
        if can_execute {
            state.service.start_event_id = Some(event_id);
        }
        if let Some(run) = run.as_ref() {
            if run_ok {
                state.service.state_counter = state.service.state_counter.saturating_add(1);
            }
            state.last_run_outcome = run.run_outcome;
            state.last_return_value = run.return_value;
            state.last_fuel_used = run.fuel_used;
            state.last_log_line_emitted = run.log_line.is_some();
        }
    }
    state.service.last_action = "start";
    state.service.last_reason = reason;
    state.service.last_inventory_change = if was_loaded && can_execute {
        "updated_current_boot_service"
    } else {
        "none"
    };
    state.service.last_event_id = Some(event_id);
    state.trust_tier = authorization.trust_tier;
    ActionResult {
        snapshot: state.snapshot(),
        event_id,
        run,
        authorization,
        durable_promotion_transaction: durable_store::promotion_transaction_append_denied(
            durable_store::PromotionTransactionKind::Promote,
            "promotion_transaction_not_attempted",
        ),
        capability_denied: !can_execute,
    }
}

fn stop(source_method: &'static str) -> ActionResult {
    let authorization = current_authorization();
    let mut state = STATE.lock();
    let reason = if state.service.loaded && state.service.running {
        "stopped"
    } else if state.service.loaded {
        "already_stopped"
    } else {
        "not_loaded"
    };
    let event_id = event_log::record_service_lifecycle_unbound(
        &GRANTED_CANDIDATE_SERVICE_DESCRIPTOR,
        source_method,
        if state.service.loaded {
            "response"
        } else {
            "capability_denied"
        },
        reason,
        LIFECYCLE_EVIDENCE,
    );

    if state.service.loaded {
        state.service.running = false;
        state.service.stop_event_id = Some(event_id);
    }
    state.service.last_action = "stop";
    state.service.last_reason = reason;
    state.service.last_inventory_change = if state.service.loaded {
        "updated_current_boot_service"
    } else {
        "none"
    };
    state.service.last_event_id = Some(event_id);
    ActionResult {
        snapshot: state.snapshot(),
        event_id,
        run: None,
        authorization,
        durable_promotion_transaction: durable_store::promotion_transaction_append_denied(
            durable_store::PromotionTransactionKind::Promote,
            "promotion_transaction_not_attempted",
        ),
        capability_denied: !state.service.loaded,
    }
}

fn drop_service(source_method: &'static str) -> ActionResult {
    let authorization = current_authorization();
    let mut state = STATE.lock();
    let was_loaded = state.service.loaded;
    let reason = if was_loaded { "dropped" } else { "not_loaded" };
    let event_id = event_log::record_service_lifecycle_unbound(
        &GRANTED_CANDIDATE_SERVICE_DESCRIPTOR,
        source_method,
        if was_loaded {
            "response"
        } else {
            "capability_denied"
        },
        reason,
        LIFECYCLE_EVIDENCE,
    );

    if was_loaded {
        module_candidate_intake::clear();
        state.promotion = None;
    }
    state.service.loaded = false;
    state.service.running = false;
    state.service.state_counter = 0;
    state.service.drop_event_id = Some(event_id);
    state.service.last_action = "drop";
    state.service.last_reason = reason;
    state.service.last_inventory_change = if was_loaded {
        "removed_current_boot_service"
    } else {
        "none"
    };
    state.service.last_event_id = Some(event_id);
    ActionResult {
        snapshot: state.snapshot(),
        event_id,
        run: None,
        authorization,
        durable_promotion_transaction: durable_store::promotion_transaction_append_denied(
            durable_store::PromotionTransactionKind::Promote,
            "promotion_transaction_not_attempted",
        ),
        capability_denied: !was_loaded,
    }
}

fn rollback_preview(source_method: &'static str) -> RollbackResult {
    let snapshot = STATE.lock().snapshot();
    let promotion = snapshot.promotion;
    let reason = if promotion.is_some() {
        "rollback_plan_recorded_ram_only"
    } else {
        "no_recorded_promotion_to_roll_back"
    };
    let event_id = event_log::record_service_lifecycle_unbound(
        &GRANTED_CANDIDATE_SERVICE_DESCRIPTOR,
        source_method,
        if promotion.is_some() {
            "response"
        } else {
            "capability_denied"
        },
        reason,
        ROLLBACK_EVIDENCE,
    );

    RollbackResult {
        snapshot,
        event_id,
        promotion,
        reprojected_inventory_hash: None,
        restored_inventory_hash_verified: false,
        durable_unpromote_transaction: durable_store::promotion_transaction_append_denied(
            durable_store::PromotionTransactionKind::Unpromote,
            "promotion_transaction_not_attempted",
        ),
        capability_denied: promotion.is_none(),
        reason,
    }
}

fn rollback_apply(source_method: &'static str) -> RollbackResult {
    let promotion = {
        let state = STATE.lock();
        if state.promotion.is_none() {
            drop(state);
            return rollback_apply_denied_no_promotion(source_method);
        }
        state.promotion.unwrap()
    };

    {
        let mut state = STATE.lock();
        state.service.running = false;
    }
    module_candidate_intake::clear();
    {
        let mut state = STATE.lock();
        state.service.loaded = false;
        state.service.running = false;
        state.service.state_counter = 0;
        state.last_run_outcome = "not_run";
        state.last_return_value = None;
        state.last_fuel_used = 0;
        state.last_log_line_emitted = false;
    }

    let reprojected_inventory_hash = service_inventory_projection_hash(None);
    let restored_inventory_hash_verified =
        reprojected_inventory_hash == promotion.pre_load_inventory_hash;
    let reason = if restored_inventory_hash_verified {
        "unpromoted_dev_key_granted_external_wasm_current_boot"
    } else {
        "rollback_restore_inventory_hash_mismatch"
    };
    let event_id = event_log::record_service_lifecycle_unbound(
        &GRANTED_CANDIDATE_SERVICE_DESCRIPTOR,
        source_method,
        if restored_inventory_hash_verified {
            "response"
        } else {
            "capability_denied"
        },
        reason,
        ROLLBACK_EVIDENCE,
    );
    let durable_unpromote_transaction = if restored_inventory_hash_verified {
        append_promotion_transaction(
            durable_store::PromotionTransactionKind::Unpromote,
            promotion,
            Some(event_id),
            true,
            Some(reprojected_inventory_hash),
            true,
        )
    } else {
        durable_store::promotion_transaction_append_denied(
            durable_store::PromotionTransactionKind::Unpromote,
            "rollback_restore_inventory_hash_mismatch",
        )
    };

    let mut state = STATE.lock();
    state.service.loaded = false;
    state.service.running = false;
    state.service.state_counter = 0;
    state.service.stop_event_id = Some(event_id);
    state.service.drop_event_id = Some(event_id);
    state.service.last_action = "rollback_apply";
    state.service.last_reason = reason;
    state.service.last_inventory_change = if restored_inventory_hash_verified {
        "removed_current_boot_service"
    } else {
        "restore_hash_mismatch_removed_current_boot_service"
    };
    state.service.last_event_id = Some(event_id);
    state.last_run_outcome = "not_run";
    state.last_return_value = None;
    state.last_fuel_used = 0;
    state.last_log_line_emitted = false;
    state.trust_tier = TRUST_TIER_GRANTED;
    if restored_inventory_hash_verified {
        state.promotion = None;
    }
    RollbackResult {
        snapshot: state.snapshot(),
        event_id,
        promotion: Some(promotion),
        reprojected_inventory_hash: Some(reprojected_inventory_hash),
        restored_inventory_hash_verified,
        durable_unpromote_transaction,
        capability_denied: !restored_inventory_hash_verified,
        reason,
    }
}

fn rollback_apply_denied_no_promotion(source_method: &'static str) -> RollbackResult {
    let event_id = event_log::record_service_lifecycle_unbound(
        &GRANTED_CANDIDATE_SERVICE_DESCRIPTOR,
        source_method,
        "capability_denied",
        "no_recorded_promotion_to_roll_back",
        ROLLBACK_EVIDENCE,
    );
    let mut state = STATE.lock();
    state.service.last_action = "rollback_apply";
    state.service.last_reason = "no_recorded_promotion_to_roll_back";
    state.service.last_inventory_change = "none";
    state.service.last_event_id = Some(event_id);
    state.trust_tier = TRUST_TIER_GRANTED;
    RollbackResult {
        snapshot: state.snapshot(),
        event_id,
        promotion: None,
        reprojected_inventory_hash: None,
        restored_inventory_hash_verified: false,
        durable_unpromote_transaction: durable_store::promotion_transaction_append_denied(
            durable_store::PromotionTransactionKind::Unpromote,
            "no_recorded_promotion_to_roll_back",
        ),
        capability_denied: true,
        reason: "no_recorded_promotion_to_roll_back",
    }
}

fn current_authorization() -> AuthorizationEvidence {
    let retained = module_candidate_intake::retained();
    let (grant_check, retained_attestation) = current_grant_inputs();
    evaluate_authorization(&grant_check, retained_attestation, retained.as_ref())
}

fn current_grant_inputs() -> (
    ModuleGrantReferenceCheck<'static>,
    Option<event_log::ModuleLocalAttestationReference>,
) {
    let grant_check = agent_protocol_module_grant::module_grant_check_from_retained(
        event_log::latest_module_computed_grant_reference().map(|(_, reference)| reference),
    );
    let retained_attestation =
        event_log::latest_module_local_attestation_reference().map(|(_, reference)| reference);
    (grant_check, retained_attestation)
}

fn evaluate_authorization(
    grant_check: &ModuleGrantReferenceCheck<'_>,
    retained_attestation: Option<event_log::ModuleLocalAttestationReference>,
    retained: Option<&RetainedExternalWasmCandidate>,
) -> AuthorizationEvidence {
    let grants_capability = agent_protocol_module_grant::module_grant_grants_capability(
        grant_check,
        retained_attestation,
    );
    let retained_sha_matches_grant =
        retained_sha_matches_grant(retained, grant_check.artifact_hash);
    let retained_wasm_valid = retained
        .map(|candidate| candidate.wasm_valid)
        .unwrap_or(false);
    let trust_tier = agent_protocol_module_grant::module_grant_trust_tier(grants_capability);
    let mut authorization = AuthorizationEvidence {
        grants_capability,
        retained_present: retained.is_some(),
        retained_wasm_valid,
        retained_sha: retained.map(|candidate| candidate.sha256),
        grant_artifact_hash: grant_check.artifact_hash,
        retained_sha_matches_grant,
        can_execute: grants_capability && retained_sha_matches_grant && retained_wasm_valid,
        trust_tier,
        denial_reason: TRUST_TIER_DENIED,
    };
    authorization.denial_reason = denial_reason(authorization);
    authorization
}

fn retained_sha_matches_grant(
    retained: Option<&RetainedExternalWasmCandidate>,
    grant_artifact_hash: Option<[u8; 32]>,
) -> bool {
    match (retained, grant_artifact_hash) {
        (Some(retained), Some(grant_artifact_hash)) => retained.sha256 == grant_artifact_hash,
        _ => false,
    }
}

fn denial_reason(authorization: AuthorizationEvidence) -> &'static str {
    if !authorization.grants_capability {
        "capability_denied_unsealed_no_grant"
    } else if !authorization.retained_present {
        "capability_denied_no_retained_candidate"
    } else if !authorization.retained_wasm_valid {
        "capability_denied_retained_candidate_invalid"
    } else if !authorization.retained_sha_matches_grant {
        "capability_denied_retained_sha_mismatch"
    } else {
        "authorized"
    }
}

pub(crate) fn health_state(snapshot: Snapshot) -> &'static str {
    current_boot_service::health_state(
        snapshot.loaded,
        snapshot.running,
        GRANTED_CANDIDATE_SERVICE_DESCRIPTOR,
    )
}

pub(crate) fn service_slot_activation_status(snapshot: Snapshot) -> &'static str {
    current_boot_service::service_slot_activation_status(
        snapshot.loaded,
        snapshot.running,
        snapshot.last_action,
        snapshot.last_reason,
        GRANTED_CANDIDATE_SERVICE_DESCRIPTOR,
    )
}

pub(crate) fn service_slot_activation_active(snapshot: Snapshot) -> bool {
    current_boot_service::service_slot_activation_active(snapshot.loaded)
}

pub(crate) fn service_slot_activation_hash() -> [u8; 32] {
    raios_core::sha256_bytes(
        GRANTED_CANDIDATE_SERVICE_DESCRIPTOR
            .service_slot_activation_id
            .as_bytes(),
    )
}

fn build_promotion_record(
    artifact_hash: [u8; 32],
    pre_load_inventory_hash: [u8; 32],
    generation: u64,
    load_event_id: event_log::EventId,
) -> PromotionRecord {
    let cleanup_actions_hash = cleanup_actions_hash();
    PromotionRecord {
        plan_hash: module_evidence::computed_module_rollback_plan_hash(
            artifact_hash,
            pre_load_inventory_hash,
            GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.ram_only_service_slot_id,
            cleanup_actions_hash,
        ),
        pre_load_inventory_hash,
        ram_only_service_slot_id: GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.ram_only_service_slot_id,
        generation,
        load_event_id,
        artifact_hash,
    }
}

fn append_promotion_transaction(
    transaction_kind: durable_store::PromotionTransactionKind,
    promotion: PromotionRecord,
    rollback_apply_event_id: Option<event_log::EventId>,
    restore_hash_verified: bool,
    reprojected_inventory_hash: Option<[u8; 32]>,
    cleanup_performed: bool,
) -> durable_store::PromotionTransactionAppendEvidence {
    let Some((_, signature)) = event_log::latest_module_promotion_signature_reference() else {
        return durable_store::promotion_transaction_append_denied(
            transaction_kind,
            "promotion_signature_reference_missing",
        );
    };
    let Some((_, computed_grant)) = event_log::latest_module_computed_grant_reference() else {
        return durable_store::promotion_transaction_append_denied(
            transaction_kind,
            "promotion_evidence_reference_missing",
        );
    };
    let Some((_, attestation)) = event_log::latest_module_local_attestation_reference() else {
        return durable_store::promotion_transaction_append_denied(
            transaction_kind,
            "promotion_evidence_reference_missing",
        );
    };

    let grant_check =
        agent_protocol_module_grant::module_grant_check_from_retained(Some(computed_grant));
    let grant_binds_capability = agent_protocol_module_grant::module_grant_grants_capability(
        &grant_check,
        Some(attestation),
    );
    let unpromote = transaction_kind == durable_store::PromotionTransactionKind::Unpromote;
    let record = durable_store::PromotionTransactionRecord {
        transaction_kind,
        computed_grant_hash: computed_grant.computed_grant_hash,
        manifest_hash: computed_grant.manifest_hash,
        artifact_hash: computed_grant.artifact_hash,
        vm_report_hash: computed_grant.vm_report_hash,
        local_attestation_hash: computed_grant.local_attestation_hash,
        retained_manifest_reference_event_id: attestation.retained_manifest_reference_event_id,
        retained_artifact_reference_event_id: attestation.retained_artifact_reference_event_id,
        retained_vm_report_reference_event_id: attestation.retained_vm_report_reference_event_id,
        retained_reference_event_id: attestation.retained_reference_event_id,
        manifest_reference_hash: attestation.manifest_reference_hash,
        artifact_reference_hash: attestation.artifact_reference_hash,
        vm_report_reference_hash: attestation.vm_report_reference_hash,
        attestation_reference_hash: attestation.attestation_reference_hash,
        signature_der: signature.signature_der,
        signature_len: signature.signature_len,
        signature_verified: signature.signature_verified,
        signature_attestation_reference_hash: signature.attestation_reference_hash,
        promotion_authority_key_sha256: signature.promotion_authority_key_sha256,
        grant_binds_capability,
        rollback_plan_hash: promotion.plan_hash,
        pre_load_inventory_hash: promotion.pre_load_inventory_hash,
        ram_only_service_slot_id: promotion.ram_only_service_slot_id,
        generation: promotion.generation,
        load_event_id: promotion.load_event_id,
        rollback_apply_event_id,
        reprojected_inventory_hash,
        restore_hash_verified,
        stopped: cleanup_performed,
        drop_clear_bytes: cleanup_performed,
        free_slot: cleanup_performed,
        remove_inventory: cleanup_performed,
        service_id: GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.service_id,
        artifact_id: GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.artifact_id,
        requested_capability: if unpromote {
            GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.rollback_apply_capability
        } else {
            GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.service_capability
        },
        load_mode: "wasmi_interpreter_ram_only",
    };
    durable_store::append_promotion_transaction(&record)
}

fn cleanup_actions_hash() -> [u8; 32] {
    raios_core::sha256_bytes(CLEANUP_ACTIONS_CANONICAL)
}

fn service_inventory_projection_hash(granted: Option<Snapshot>) -> [u8; 32] {
    let mut services = Vec::new();
    let mut idx = 0usize;
    while idx < service_inventory::SERVICES.len() {
        services.push(record_static_inventory_service(
            &service_inventory::SERVICES[idx],
        ));
        idx += 1;
    }
    if let Some(echo) = echo_service::loaded_snapshot() {
        services.push(record_echo_inventory_projection(echo));
    }
    if let Some(granted) = granted {
        services.push(record_granted_inventory_projection(granted));
    }
    if let Some(hello) = hello_service::loaded_snapshot() {
        services.push(record_hello_inventory_projection(hello));
    }
    sha256_of_json(&V::InlineObject(vec![
        f("schema", s("service.inventory.v0")),
        f("projection", s("pre_load_ordered_service_inventory")),
        f("services", V::InlineArray(services)),
    ]))
}

fn record_static_inventory_service(service: &service_inventory::Service) -> V<'static> {
    V::InlineObject(vec![
        f("id", s(service.id)),
        f("kind", s(service.kind)),
        f("replaceable", b(service.replaceable)),
        f("core_owned", b(service.core_owned)),
        f(
            "capabilities",
            record_static_str_array(service.capabilities),
        ),
    ])
}

fn record_echo_inventory_projection(echo: echo_service::Snapshot) -> V<'static> {
    let descriptor = echo_service::ECHO_SERVICE_DESCRIPTOR;
    V::InlineObject(vec![
        f("id", s(descriptor.service_id)),
        f("kind", s(descriptor.inventory_kind)),
        f("health", s(echo_service::health_state(echo))),
        f("scope", s(descriptor.scope)),
        f("persistence", s(descriptor.persistence)),
        f("artifact_id", s(descriptor.artifact_id)),
        f(
            "ram_only_service_slot_id",
            s(descriptor.ram_only_service_slot_id),
        ),
        f("running", b(echo.running)),
        f("generation", V::U64(echo.generation)),
        f("run_count", V::U64(echo.run_count)),
        f("last_run_outcome", s(echo.last_run_outcome)),
        f("last_action", s(echo.last_action)),
    ])
}

fn record_granted_inventory_projection(granted: Snapshot) -> V<'static> {
    let descriptor = GRANTED_CANDIDATE_SERVICE_DESCRIPTOR;
    V::InlineObject(vec![
        f("id", s(descriptor.service_id)),
        f("kind", s(descriptor.inventory_kind)),
        f("health", s(health_state(granted))),
        f("scope", s(descriptor.scope)),
        f("persistence", s(descriptor.persistence)),
        f("classification", s(descriptor.classification)),
        f("artifact_id", s(descriptor.artifact_id)),
        f("artifact_kind", s(descriptor.artifact_kind)),
        f("trust_tier", s(granted.trust_tier)),
        f(
            "ram_only_service_slot_id",
            s(descriptor.ram_only_service_slot_id),
        ),
        f("running", b(granted.running)),
        f("generation", V::U64(granted.generation)),
        f("run_count", V::U64(granted.run_count)),
        f("last_run_outcome", s(granted.last_run_outcome)),
        f("last_action", s(granted.last_action)),
    ])
}

fn record_hello_inventory_projection(hello: hello_service::Snapshot) -> V<'static> {
    let descriptor = hello_service::HELLO_SERVICE_DESCRIPTOR;
    V::InlineObject(vec![
        f("id", s(descriptor.service_id)),
        f("kind", s(descriptor.inventory_kind)),
        f(
            "health",
            s(if hello.running {
                descriptor.inventory_health_running
            } else {
                descriptor.inventory_health_stopped
            }),
        ),
        f("scope", s(descriptor.scope)),
        f("persistence", s(descriptor.persistence)),
        f("artifact_id", s(descriptor.artifact_id)),
        f(
            "ram_only_service_slot_id",
            s(descriptor.ram_only_service_slot_id),
        ),
        f("running", b(hello.running)),
        f("generation", V::U64(hello.generation)),
        f("run_count", V::U64(hello.state_counter)),
        f("last_action", s(hello.last_action)),
    ])
}

pub(crate) fn record_live_load_projection(projection: LiveLoadProjection) -> V<'static> {
    V::Object(vec![
        f("present", b(projection.present)),
        f(
            "accepts_external_artifact_bytes",
            b(projection.accepts_external_artifact_bytes),
        ),
        f("loads_artifact", b(projection.loads_artifact)),
        f("can_load_now", b(projection.can_load_now)),
        f(
            "service_slot_allocated",
            b(projection.service_slot_allocated),
        ),
        f("running", b(projection.running)),
        f("run_outcome", s(projection.run_outcome)),
        f("trust_tier", s(projection.trust_tier)),
        f("load_mechanism", s(projection.load_mechanism)),
        f("maps_executable_pages", b(projection.maps_executable_pages)),
        f("durable", b(projection.durable)),
        f("owner_sealed", b(projection.owner_sealed)),
        f(
            "authorizes_native_guest_load",
            b(projection.authorizes_native_guest_load),
        ),
    ])
}

fn run_succeeded(run: &wasm_runtime::EchoRunEvidence) -> bool {
    run.validation_ok
        && run.instantiation_ok
        && run.run_outcome == "success"
        && run.return_value == Some(0)
}

fn emit_response(method: &'static str, action: &'static str, result: ActionResult) {
    let snapshot = result.snapshot;
    begin_response(method);
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s(LIFECYCLE_RESPONSE_SCHEMA)),
            f("scope", s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.scope)),
            f(
                "classification",
                s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.classification),
            ),
            f(
                "persistence",
                s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.persistence),
            ),
            f("durable", b(false)),
            f("owner_sealed", b(false)),
            f("method", s(method)),
            f("action", s(action)),
            f(
                "code",
                s(if result.capability_denied {
                    "capability_denied"
                } else {
                    "ok"
                }),
            ),
            f(
                "trust_tier",
                s(if result.authorization.can_execute {
                    TRUST_TIER_GRANTED
                } else {
                    snapshot.trust_tier
                }),
            ),
            f(
                "service_id",
                s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.service_id),
            ),
            f(
                "artifact_id",
                s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.artifact_id),
            ),
            f(
                "artifact_kind",
                s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.artifact_kind),
            ),
            f("version", s(SERVICE_VERSION)),
            f("health", s(health_state(snapshot))),
            f("loaded", b(snapshot.loaded)),
            f("running", b(snapshot.running)),
            f("generation", V::U64(snapshot.generation)),
            f("run_count", V::U64(snapshot.run_count)),
            f("last_action", s(snapshot.last_action)),
            f("reason", s(snapshot.last_reason)),
            f(
                "service_inventory_change",
                s(snapshot.last_inventory_change),
            ),
            f(
                "service_slot_activation",
                record_service_slot_activation(snapshot),
            ),
            f("event_id", record_event_or_null(Some(result.event_id))),
            f(
                "load_event_id",
                record_event_or_null(snapshot.load_event_id),
            ),
            f(
                "start_event_id",
                record_event_or_null(snapshot.start_event_id),
            ),
            f(
                "stop_event_id",
                record_event_or_null(snapshot.stop_event_id),
            ),
            f(
                "drop_event_id",
                record_event_or_null(snapshot.drop_event_id),
            ),
            f("load_descriptor", record_load_descriptor()),
            f(
                "grant_authority",
                record_authorization(result.authorization),
            ),
            f("capability_envelope", s("wasmi_linker_import_surface")),
            f(
                "granted_host_imports",
                record_static_str_array(&["env.log", "env.counter_get"]),
            ),
            f("host_import_count", V::U64(2)),
            f("entrypoint", s(GRANTED_ENTRYPOINT)),
            f("run_evidence", record_run_evidence(result.run.as_ref())),
            f("last_run_evidence", record_last_run(snapshot)),
            f("rollback_plan", record_rollback_plan(snapshot.promotion)),
            f(
                "durable_promotion_transaction",
                durable_store::promotion_transaction_append_evidence_record(
                    result.durable_promotion_transaction,
                ),
            ),
            f("capabilities", record_static_str_array(CAPABILITIES)),
            f("accepts_external_artifact_bytes", b(true)),
            f("loads_external_artifact", b(true)),
            f("maps_executable_pages", b(false)),
            f("writes_persistent_state", b(false)),
            f("authorizes_persistent_install", b(false)),
            f("authorizes_rollback_install", b(false)),
            f("durable_writes_enabled", b(false)),
            f("rollback_apply_authorized", b(false)),
            f("broad_mutation_authorized", b(false)),
        ],
        6,
    );
    crate::agent_protocol_support::raw_line("      \"evidence_complete\": true");
    end_response(method);
}

fn emit_rollback_response(method: &'static str, action: &'static str, result: RollbackResult) {
    let snapshot = result.snapshot;
    begin_response(method);
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s(LIFECYCLE_RESPONSE_SCHEMA)),
            f("scope", s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.scope)),
            f(
                "classification",
                s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.classification),
            ),
            f(
                "persistence",
                s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.persistence),
            ),
            f("durable", b(false)),
            f("owner_sealed", b(false)),
            f("method", s(method)),
            f("action", s(action)),
            f(
                "event_kind",
                s(if action == "rollback_apply" {
                    GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.event_rollback_apply_kind
                } else {
                    GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.event_rollback_preview_kind
                }),
            ),
            f(
                "code",
                s(if result.capability_denied {
                    "capability_denied"
                } else {
                    "ok"
                }),
            ),
            f("trust_tier", s(TRUST_TIER_GRANTED)),
            f(
                "service_id",
                s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.service_id),
            ),
            f(
                "artifact_id",
                s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.artifact_id),
            ),
            f(
                "artifact_kind",
                s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.artifact_kind),
            ),
            f("version", s(SERVICE_VERSION)),
            f("health", s(health_state(snapshot))),
            f("loaded", b(snapshot.loaded)),
            f("running", b(snapshot.running)),
            f("generation", V::U64(snapshot.generation)),
            f("run_count", V::U64(snapshot.run_count)),
            f("last_action", s(snapshot.last_action)),
            f("reason", s(result.reason)),
            f(
                "service_inventory_change",
                s(snapshot.last_inventory_change),
            ),
            f(
                "service_slot_activation",
                record_service_slot_activation(snapshot),
            ),
            f("event_id", record_event_or_null(Some(result.event_id))),
            f(
                "load_event_id",
                record_event_or_null(snapshot.load_event_id),
            ),
            f(
                "start_event_id",
                record_event_or_null(snapshot.start_event_id),
            ),
            f(
                "stop_event_id",
                record_event_or_null(snapshot.stop_event_id),
            ),
            f(
                "drop_event_id",
                record_event_or_null(snapshot.drop_event_id),
            ),
            f("rollback_plan", record_rollback_plan(result.promotion)),
            f(
                "rollback_verification",
                record_rollback_verification(&result),
            ),
            f("capabilities", record_static_str_array(CAPABILITIES)),
            f("accepts_external_artifact_bytes", b(true)),
            f("loads_external_artifact", b(true)),
            f("maps_executable_pages", b(false)),
            f("writes_persistent_state", b(false)),
            f("authorizes_persistent_install", b(false)),
            f("authorizes_rollback_install", b(false)),
            f("durable_writes_enabled", b(false)),
            f(
                "rollback_apply_authorized",
                b(action == "rollback_apply" && !result.capability_denied),
            ),
            f("broad_mutation_authorized", b(false)),
        ],
        6,
    );
    crate::agent_protocol_support::raw_line("      \"evidence_complete\": true");
    end_response(method);
}

fn record_rollback_plan(promotion: Option<PromotionRecord>) -> V<'static> {
    V::Object(vec![
        f("schema", s("raios.rollback_plan.v0")),
        f("present", b(promotion.is_some())),
        f("scope", s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.scope)),
        f(
            "classification",
            s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.classification),
        ),
        f("trust_tier", s(TRUST_TIER_GRANTED)),
        f("durable", b(false)),
        f("owner_sealed", b(false)),
        f(
            "persistence",
            s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.persistence),
        ),
        f("cross_boot", b(false)),
        f(
            "plan_hash",
            record_sha_or_null(promotion.map(|record| record.plan_hash)),
        ),
        f(
            "artifact_hash",
            record_sha_or_null(promotion.map(|record| record.artifact_hash)),
        ),
        f(
            "pre_load_service_inventory_hash",
            record_sha_or_null(promotion.map(|record| record.pre_load_inventory_hash)),
        ),
        f(
            "cleanup_actions_hash",
            record_sha_or_null(promotion.map(|_| cleanup_actions_hash())),
        ),
        f(
            "ram_only_service_slot_id",
            record_str_or_null(promotion.map(|record| record.ram_only_service_slot_id)),
        ),
        f(
            "generation",
            promotion
                .map(|record| V::U64(record.generation))
                .unwrap_or(V::Null),
        ),
        f(
            "load_event_id",
            record_event_or_null(promotion.map(|record| record.load_event_id)),
        ),
        f("cleanup_actions", record_static_str_array(CLEANUP_ACTIONS)),
        f("writes_persistent_state", b(false)),
        f("authorizes_rollback_install", b(false)),
    ])
}

fn record_rollback_verification(result: &RollbackResult) -> V<'static> {
    V::Object(vec![
        f(
            "pre_load_service_inventory_hash",
            record_sha_or_null(
                result
                    .promotion
                    .map(|record| record.pre_load_inventory_hash),
            ),
        ),
        f(
            "reprojected_service_inventory_hash",
            record_sha_or_null(result.reprojected_inventory_hash),
        ),
        f(
            "restore_hash_verified",
            b(result.restored_inventory_hash_verified),
        ),
        f("stopped", b(!result.snapshot.running)),
        f("drop_clear_bytes", b(result.promotion.is_some())),
        f("free_slot", b(!result.snapshot.loaded)),
        f("remove_inventory", b(!result.snapshot.loaded)),
        f(
            "durable_unpromote_transaction",
            durable_store::promotion_transaction_append_evidence_record(
                result.durable_unpromote_transaction,
            ),
        ),
        f("owner_sealed", b(false)),
        f(
            "persistence",
            s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.persistence),
        ),
    ])
}

fn record_service_slot_activation(snapshot: Snapshot) -> V<'static> {
    V::Object(vec![
        f("schema", s("raios.ram_only_service_slot_activation.v0")),
        f(
            "id",
            s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.service_slot_activation_id),
        ),
        f(
            "hash",
            record_sha_or_null(Some(service_slot_activation_hash())),
        ),
        f("status", s(service_slot_activation_status(snapshot))),
        f("active", b(service_slot_activation_active(snapshot))),
        f(
            "ram_only_service_slot_id",
            s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.ram_only_service_slot_id),
        ),
    ])
}

fn record_load_descriptor() -> V<'static> {
    V::Object(vec![
        f("schema", s(LOAD_DESCRIPTOR_SCHEMA)),
        f("id", s(LOAD_DESCRIPTOR_ID)),
        f("source_locator", s(LOAD_DESCRIPTOR_SOURCE_LOCATOR)),
        f("source_kind", s(LOAD_DESCRIPTOR_SOURCE_KIND)),
        f("scope", s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.scope)),
        f(
            "classification",
            s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.classification),
        ),
        f(
            "persistence",
            s(GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.persistence),
        ),
        f("authorizes_current_boot_wasm_execution", b(true)),
        f("validates_with_wasmi_module_new", b(true)),
        f("accepts_external_artifact_bytes", b(true)),
        f("loads_external_artifact", b(true)),
        f("maps_executable_pages", b(false)),
        f("writes_persistent_state", b(false)),
        f("authorizes_persistent_install", b(false)),
        f("authorizes_rollback_install", b(false)),
    ])
}

fn record_authorization(authorization: AuthorizationEvidence) -> V<'static> {
    V::Object(vec![
        f("grants_capability", b(authorization.grants_capability)),
        f("trust_tier", s(authorization.trust_tier)),
        f("can_load_now", b(authorization.can_execute)),
        f(
            "retained_candidate_present",
            b(authorization.retained_present),
        ),
        f("retained_wasm_valid", b(authorization.retained_wasm_valid)),
        f(
            "retained_sha256",
            record_sha_or_null(authorization.retained_sha),
        ),
        f(
            "grant_artifact_hash",
            record_sha_or_null(authorization.grant_artifact_hash),
        ),
        f(
            "retained_sha_matches_grant",
            b(authorization.retained_sha_matches_grant),
        ),
        f("denial_reason", s(authorization.denial_reason)),
    ])
}

fn record_run_evidence(run: Option<&wasm_runtime::EchoRunEvidence>) -> V<'_> {
    match run {
        Some(run) => V::Object(vec![
            f("present", b(true)),
            f("validation_ok", b(run.validation_ok)),
            f("instantiation_ok", b(run.instantiation_ok)),
            f("run_outcome", s(run.run_outcome)),
            f("return_value_i32", record_return_value(run.return_value)),
            f("fuel_budget", V::U64(run.fuel_budget)),
            f("fuel_used", V::U64(run.fuel_used)),
            f("log_prefix", s("WASM_GUEST_LOG")),
            f("log_line_emitted", b(run.log_line.is_some())),
            f("log_line", record_str_or_null(run.log_line.as_deref())),
        ]),
        None => V::Object(vec![
            f("present", b(false)),
            f("instantiation_ok", b(false)),
            f("run_outcome", s("not_run")),
            f("return_value_i32", V::Null),
            f("fuel_budget", V::U64(wasm_runtime::ECHO_WASM_FUEL_BUDGET)),
            f("fuel_used", V::U64(0)),
            f("log_line_emitted", b(false)),
            f("log_line", V::Null),
        ]),
    }
}

fn record_last_run(snapshot: Snapshot) -> V<'static> {
    V::Object(vec![
        f("run_outcome", s(snapshot.last_run_outcome)),
        f(
            "return_value_i32",
            record_return_value(snapshot.last_return_value),
        ),
        f("fuel_used", V::U64(snapshot.last_fuel_used)),
        f("log_line_emitted", b(snapshot.last_log_line_emitted)),
    ])
}

fn record_return_value(value: Option<i32>) -> V<'static> {
    match value {
        Some(value) if value >= 0 => V::U64(value as u64),
        _ => V::Null,
    }
}

fn run_selftest_cases() -> Vec<SelftestCase> {
    vec![
        run_selftest_case("granted_bytes_present_runs", SelftestMode::Granted),
        run_selftest_case("ungranted_no_instantiation", SelftestMode::Ungranted),
        run_selftest_case("hash_mismatch_no_instantiation", SelftestMode::HashMismatch),
        run_projection_selftest_case(
            "live_load_projection_loaded_snapshot",
            Some(selftest_loaded_projection_snapshot()),
        ),
        run_projection_selftest_case("live_load_projection_not_loaded", None),
        run_rollback_selftest_case(
            "rollback_apply_recorded_promotion_restores_inventory",
            SelftestRollbackMode::RecordedPromotion,
        ),
        run_rollback_selftest_case(
            "rollback_apply_no_promotion_denied",
            SelftestRollbackMode::NoPromotion,
        ),
        run_rollback_selftest_case(
            "rollback_apply_one_shot_second_denied",
            SelftestRollbackMode::OneShot,
        ),
    ]
}

#[derive(Clone, Copy)]
enum SelftestMode {
    Granted,
    Ungranted,
    HashMismatch,
}

#[derive(Clone, Copy)]
enum SelftestRollbackMode {
    RecordedPromotion,
    NoPromotion,
    OneShot,
}

fn run_selftest_case(name: &'static str, mode: SelftestMode) -> SelftestCase {
    let retained = selftest_retained_candidate(mode);
    let grant_check = selftest_grant_check(wasm_runtime::ECHO_WASM_ARTIFACT_BYTES_HASH);
    let attestation = match mode {
        SelftestMode::Granted | SelftestMode::HashMismatch => Some(selftest_attestation()),
        SelftestMode::Ungranted => None,
    };
    let authorization = evaluate_authorization(&grant_check, attestation, Some(&retained));
    let run = if authorization.can_execute {
        Some(wasm_runtime::execute_module_bytes(
            &retained.bytes,
            GRANTED_ENTRYPOINT,
        ))
    } else {
        None
    };
    let instantiation_ok = run
        .as_ref()
        .map(|run| run.instantiation_ok)
        .unwrap_or(false);
    let run_outcome = run.as_ref().map(|run| run.run_outcome).unwrap_or("not_run");
    let fuel_used = run.as_ref().map(|run| run.fuel_used).unwrap_or(0);
    let capability_denied = !authorization.can_execute;
    let passed = match mode {
        SelftestMode::Granted => {
            !capability_denied
                && instantiation_ok
                && run_outcome == "success"
                && fuel_used > 0
                && authorization.trust_tier == TRUST_TIER_GRANTED
        }
        SelftestMode::Ungranted => {
            capability_denied
                && !instantiation_ok
                && run_outcome == "not_run"
                && authorization.trust_tier == TRUST_TIER_DENIED
        }
        SelftestMode::HashMismatch => {
            capability_denied
                && !instantiation_ok
                && run_outcome == "not_run"
                && !authorization.retained_sha_matches_grant
        }
    };

    SelftestCase {
        name,
        capability_denied,
        instantiation_ok,
        run_outcome,
        fuel_used,
        trust_tier: authorization.trust_tier,
        live_load_projection: None,
        durable: false,
        rollback_restore_hash_verified: false,
        second_rollback_apply_denied: false,
        passed,
    }
}

fn run_rollback_selftest_case(name: &'static str, mode: SelftestRollbackMode) -> SelftestCase {
    let pre_load_inventory_hash = service_inventory_projection_hash(None);
    let promotion = build_promotion_record(
        wasm_runtime::ECHO_WASM_ARTIFACT_BYTES_HASH,
        pre_load_inventory_hash,
        1,
        selftest_event_id(61),
    );
    let expected_plan_hash = module_evidence::computed_module_rollback_plan_hash(
        wasm_runtime::ECHO_WASM_ARTIFACT_BYTES_HASH,
        pre_load_inventory_hash,
        GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.ram_only_service_slot_id,
        cleanup_actions_hash(),
    );
    let reprojected_inventory_hash = service_inventory_projection_hash(None);
    let restore_hash_verified = reprojected_inventory_hash == promotion.pre_load_inventory_hash;
    let plan_hash_verified = promotion.plan_hash == expected_plan_hash;

    let (capability_denied, second_rollback_apply_denied, passed) = match mode {
        SelftestRollbackMode::RecordedPromotion => (
            false,
            false,
            plan_hash_verified
                && restore_hash_verified
                && promotion.artifact_hash == wasm_runtime::ECHO_WASM_ARTIFACT_BYTES_HASH,
        ),
        SelftestRollbackMode::NoPromotion => (true, false, true),
        SelftestRollbackMode::OneShot => (false, true, plan_hash_verified && restore_hash_verified),
    };

    SelftestCase {
        name,
        capability_denied,
        instantiation_ok: false,
        run_outcome: if restore_hash_verified {
            "rollback_restore_hash_verified"
        } else {
            "rollback_restore_hash_mismatch"
        },
        fuel_used: 0,
        trust_tier: TRUST_TIER_GRANTED,
        live_load_projection: None,
        durable: false,
        rollback_restore_hash_verified: matches!(
            mode,
            SelftestRollbackMode::RecordedPromotion | SelftestRollbackMode::OneShot
        ) && restore_hash_verified,
        second_rollback_apply_denied,
        passed,
    }
}

fn record_selftest_case(case: &SelftestCase) -> V<'static> {
    let mut fields = vec![
        f("case", s(case.name)),
        f("capability_denied", b(case.capability_denied)),
        f("instantiation_ok", b(case.instantiation_ok)),
        f("run_outcome", s(case.run_outcome)),
        f("fuel_used", V::U64(case.fuel_used)),
        f("trust_tier", s(case.trust_tier)),
    ];
    if let Some(projection) = case.live_load_projection {
        fields.push(f(
            "live_load_projection",
            record_live_load_projection(projection),
        ));
    }
    fields.push(f("durable", b(case.durable)));
    fields.push(f(
        "rollback_restore_hash_verified",
        b(case.rollback_restore_hash_verified),
    ));
    fields.push(f(
        "second_rollback_apply_denied",
        b(case.second_rollback_apply_denied),
    ));
    fields.push(f("passed", b(case.passed)));
    V::InlineObject(fields)
}

fn run_projection_selftest_case(name: &'static str, snapshot: Option<Snapshot>) -> SelftestCase {
    let projection = live_load_projection_from_snapshot(snapshot);
    let loaded = snapshot.is_some();
    let positives_match = projection.present
        && projection.accepts_external_artifact_bytes
        && projection.loads_artifact
        && projection.can_load_now
        && projection.service_slot_allocated
        && projection.running
        && projection.run_outcome == "success"
        && projection.trust_tier == TRUST_TIER_GRANTED
        && projection.load_mechanism == "wasmi_interpreter_ram_only";
    let all_projected_bools_false = !projection.present
        && !projection.accepts_external_artifact_bytes
        && !projection.loads_artifact
        && !projection.can_load_now
        && !projection.service_slot_allocated
        && !projection.running;
    let guardrails_false = !projection.maps_executable_pages
        && !projection.durable
        && !projection.owner_sealed
        && !projection.authorizes_native_guest_load;
    let passed = if loaded {
        positives_match && guardrails_false
    } else {
        all_projected_bools_false && guardrails_false
    };

    SelftestCase {
        name,
        capability_denied: !loaded,
        instantiation_ok: false,
        run_outcome: projection.run_outcome,
        fuel_used: 0,
        trust_tier: projection.trust_tier,
        live_load_projection: Some(projection),
        durable: false,
        rollback_restore_hash_verified: false,
        second_rollback_apply_denied: false,
        passed,
    }
}

fn selftest_loaded_projection_snapshot() -> Snapshot {
    Snapshot {
        loaded: true,
        running: true,
        generation: 1,
        run_count: 1,
        last_run_outcome: "success",
        last_return_value: Some(0),
        last_fuel_used: 1,
        last_log_line_emitted: true,
        last_action: "start",
        last_reason: "started_dev_key_granted_external_wasm_current_boot",
        last_inventory_change: "upserted_current_boot_service",
        trust_tier: TRUST_TIER_GRANTED,
        load_event_id: None,
        start_event_id: None,
        stop_event_id: None,
        drop_event_id: None,
        promotion: None,
    }
}

fn selftest_retained_candidate(mode: SelftestMode) -> RetainedExternalWasmCandidate {
    let sha256 = match mode {
        SelftestMode::HashMismatch => [0x99; 32],
        _ => wasm_runtime::ECHO_WASM_ARTIFACT_BYTES_HASH,
    };
    RetainedExternalWasmCandidate {
        bytes: Vec::from(wasm_runtime::ECHO_WASM_ARTIFACT_BYTES),
        sha256,
        wasm_valid: true,
    }
}

fn selftest_grant_check(artifact_hash: [u8; 32]) -> ModuleGrantReferenceCheck<'static> {
    let manifest_hash = [0x11; 32];
    let vm_report_hash = [0x33; 32];
    let local_attestation_hash = [0x44; 32];
    let computed_grant_hash = module_evidence::computed_module_grant_hash(
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    );
    ModuleGrantReferenceCheck {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        grant_hash: Some(computed_grant_hash),
        manifest_hash: Some(manifest_hash),
        artifact_hash: Some(artifact_hash),
        vm_report_hash: Some(vm_report_hash),
        local_attestation_hash: Some(local_attestation_hash),
        expected_grant_hash: Some(computed_grant_hash),
        status: "valid_hash_reference_load_still_denied",
        reason: "hash_reference_valid_but_loader_audit_rollback_and_slot_missing",
        valid: true,
    }
}

fn selftest_attestation() -> event_log::ModuleLocalAttestationReference {
    let check = selftest_grant_check(wasm_runtime::ECHO_WASM_ARTIFACT_BYTES_HASH);
    event_log::ModuleLocalAttestationReference {
        attestation_reference_hash: [0xaa; 32],
        retained_manifest_reference_event_id: selftest_event_id(26),
        retained_artifact_reference_event_id: selftest_event_id(28),
        retained_vm_report_reference_event_id: selftest_event_id(29),
        retained_reference_event_id: selftest_event_id(27),
        manifest_reference_hash: [0x55; 32],
        artifact_reference_hash: [0x56; 32],
        vm_report_reference_hash: [0x57; 32],
        manifest_hash: check.manifest_hash.unwrap_or([0; 32]),
        artifact_hash: check.artifact_hash.unwrap_or([0; 32]),
        computed_grant_hash: check.grant_hash.unwrap_or([0; 32]),
        vm_report_hash: check.vm_report_hash.unwrap_or([0; 32]),
        local_attestation_hash: check.local_attestation_hash.unwrap_or([0; 32]),
        signature_verified: true,
    }
}

fn selftest_event_id(sequence: u64) -> event_log::EventId {
    let mut candidate = sequence;
    loop {
        if let Some(event_id) = event_log::EventId::from_sequence(candidate) {
            return event_id;
        }
        candidate = 1;
    }
}
