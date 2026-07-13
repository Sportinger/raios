use alloc::{vec, vec::Vec};

use crate::agent_protocol_support::{
    emit_inline_record_object, emit_inline_record_object_fragment,
    emit_record_fields_trailing_comma, emit_record_object, emit_record_property_line,
    record_bool as b, record_event_or_null, record_false as no, record_field as f,
    record_inline as inline, record_null as null, record_object as object, record_sha as sha,
    record_str as s, record_str_or_null,
};
use crate::{
    agent_protocol_module_reference::emit_evidence_v1_response, agent_protocol_module_types::*,
    agent_protocol_support::*, event_log,
};
use raios_core::{
    evidence_response::{self as ev, SelftestFacts},
    module_loader_allocator_projection::{
        project_allocator_denial, AllocatorProjectionInput, LoaderAllocatorDisposition,
        LoaderAllocatorEvidenceInput, LoaderAllocatorEvidenceStatus,
    },
    record::{Field, Value as V},
};

fn allocator_evidence<'a>(
    status: &'static str,
    reason: &'a str,
    source_event_id: Option<event_log::EventId>,
    facts: Vec<Field<'a>>,
) -> LoaderAllocatorEvidenceInput<'a> {
    LoaderAllocatorEvidenceInput {
        status: match status {
            "available" => LoaderAllocatorEvidenceStatus::Verified,
            "missing" => LoaderAllocatorEvidenceStatus::Missing,
            "rejected" => LoaderAllocatorEvidenceStatus::Rejected,
            _ => LoaderAllocatorEvidenceStatus::Unavailable,
        },
        status_detail: status,
        reason,
        source_event_sequence: source_event_id.map(event_log::EventId::sequence),
        facts,
        disposition: if method_eq(status, "available") {
            LoaderAllocatorDisposition::Satisfied
        } else {
            LoaderAllocatorDisposition::Blocked
        },
    }
}

pub(crate) fn emit_module_service_slot_allocator() {
    let retained = event_log::latest_module_service_slot_reservation();
    let retained_event_id = retained.as_ref().map(|(event_id, _)| *event_id);
    let allocator_runtime_source_evidence = module_service_slot_allocator_runtime_source_evidence(
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0],
        retained_event_id,
        None,
    );
    let allocator_runtime_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_fact_source_evidence(
            allocator_runtime_source_evidence,
        );
    let registry_binding_source_evidence = module_service_slot_allocator_bound_fact_source_evidence(
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1],
        retained_event_id,
        Some(allocator_runtime_source_evidence_event_id),
        allocator_runtime_source_evidence,
        "service_slot_registry_binding_available",
    );
    let registry_binding_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_fact_source_evidence(
            registry_binding_source_evidence,
        );
    let health_state_source_evidence = module_service_slot_allocator_bound_fact_source_evidence(
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2],
        retained_event_id,
        Some(allocator_runtime_source_evidence_event_id),
        allocator_runtime_source_evidence,
        "service_health_state_model_available",
    );
    let health_state_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_fact_source_evidence(
            health_state_source_evidence,
        );
    let unload_cleanup_source_evidence = module_service_slot_allocator_bound_fact_source_evidence(
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3],
        retained_event_id,
        Some(allocator_runtime_source_evidence_event_id),
        allocator_runtime_source_evidence,
        "service_unload_cleanup_plan_available",
    );
    let unload_cleanup_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_fact_source_evidence(
            unload_cleanup_source_evidence,
        );
    let allocator_runtime_observed_source_evidence =
        event_log::latest_module_service_slot_allocator_fact_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0].source_fact_locator,
        )
        .unwrap_or((
            allocator_runtime_source_evidence_event_id,
            allocator_runtime_source_evidence,
        ));
    let registry_binding_observed_source_evidence =
        event_log::latest_module_service_slot_allocator_fact_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1].source_fact_locator,
        )
        .unwrap_or((
            registry_binding_source_evidence_event_id,
            registry_binding_source_evidence,
        ));
    let health_state_observed_source_evidence =
        event_log::latest_module_service_slot_allocator_fact_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2].source_fact_locator,
        )
        .unwrap_or((
            health_state_source_evidence_event_id,
            health_state_source_evidence,
        ));
    let unload_cleanup_observed_source_evidence =
        event_log::latest_module_service_slot_allocator_fact_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3].source_fact_locator,
        )
        .unwrap_or((
            unload_cleanup_source_evidence_event_id,
            unload_cleanup_source_evidence,
        ));
    let durable_audit_source_evidence =
        module_service_slot_allocator_bound_prerequisite_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0],
            retained.is_some(),
            allocator_runtime_observed_source_evidence,
            registry_binding_observed_source_evidence,
            health_state_observed_source_evidence,
            unload_cleanup_observed_source_evidence,
            "durable_audit_write_available",
        );
    let durable_audit_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_prerequisite_source_evidence(
            durable_audit_source_evidence,
        );
    let rollback_install_source_evidence =
        module_service_slot_allocator_bound_prerequisite_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1],
            retained.is_some(),
            allocator_runtime_observed_source_evidence,
            registry_binding_observed_source_evidence,
            health_state_observed_source_evidence,
            unload_cleanup_observed_source_evidence,
            "rollback_plan_install_available",
        );
    let rollback_install_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_prerequisite_source_evidence(
            rollback_install_source_evidence,
        );
    let module_loader_source_evidence =
        module_service_slot_allocator_module_loader_prerequisite_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2],
            retained.is_some(),
            allocator_runtime_observed_source_evidence,
            registry_binding_observed_source_evidence,
            health_state_observed_source_evidence,
            unload_cleanup_observed_source_evidence,
            durable_audit_source_evidence,
            rollback_install_source_evidence,
        );
    let module_loader_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_prerequisite_source_evidence(
            module_loader_source_evidence,
        );
    let durable_audit_observed_source_evidence =
        event_log::latest_module_service_slot_allocator_prerequisite_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0].source_fact_locator,
        )
        .unwrap_or((
            durable_audit_source_evidence_event_id,
            durable_audit_source_evidence,
        ));
    let rollback_install_observed_source_evidence =
        event_log::latest_module_service_slot_allocator_prerequisite_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1].source_fact_locator,
        )
        .unwrap_or((
            rollback_install_source_evidence_event_id,
            rollback_install_source_evidence,
        ));
    let module_loader_observed_source_evidence =
        event_log::latest_module_service_slot_allocator_prerequisite_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2].source_fact_locator,
        )
        .unwrap_or((
            module_loader_source_evidence_event_id,
            module_loader_source_evidence,
        ));
    let allocator_authority_source_evidence =
        module_service_slot_allocator_authority_source_evidence(
            retained.is_some(),
            allocator_runtime_observed_source_evidence,
            registry_binding_observed_source_evidence,
            health_state_observed_source_evidence,
            unload_cleanup_observed_source_evidence,
            durable_audit_observed_source_evidence,
            rollback_install_observed_source_evidence,
            module_loader_observed_source_evidence,
        );
    let allocator_authority_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_authority_source_evidence(
            allocator_authority_source_evidence,
        );
    let load_gate_binding = event_log::module_load_gate_binding_snapshot();
    let allocation_intent_source_evidence = module_service_slot_allocation_intent_source_evidence(
        load_gate_binding,
        retained.is_some(),
        (
            allocator_authority_source_evidence_event_id,
            allocator_authority_source_evidence,
        ),
    );
    let allocation_intent_source_evidence_event_id =
        event_log::record_module_service_slot_allocation_intent_source_evidence(
            allocation_intent_source_evidence,
        );
    let policy_decision_source_evidence = module_service_slot_authority_input_source_evidence(
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0],
        load_gate_binding,
        (
            allocation_intent_source_evidence_event_id,
            allocation_intent_source_evidence,
        ),
        MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SCHEMA,
        Some(allocation_intent_source_evidence_event_id),
        allocation_intent_source_evidence.intent_present
            && allocation_intent_source_evidence.source_chain_complete,
    );
    let policy_decision_source_evidence_event_id =
        event_log::record_module_service_slot_authority_input_source_evidence(
            policy_decision_source_evidence,
        );
    let registry_write_authority_source_evidence =
        module_service_slot_authority_input_source_evidence(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[1],
            load_gate_binding,
            (
                allocation_intent_source_evidence_event_id,
                allocation_intent_source_evidence,
            ),
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0].schema,
            Some(policy_decision_source_evidence_event_id),
            policy_decision_source_evidence.input_present
                && policy_decision_source_evidence.source_chain_complete,
        );
    let registry_write_authority_source_evidence_event_id =
        event_log::record_module_service_slot_authority_input_source_evidence(
            registry_write_authority_source_evidence,
        );
    let loader_runtime_contract_source_evidence =
        module_service_slot_authority_input_source_evidence(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[2],
            load_gate_binding,
            (
                allocation_intent_source_evidence_event_id,
                allocation_intent_source_evidence,
            ),
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[1].schema,
            Some(registry_write_authority_source_evidence_event_id),
            registry_write_authority_source_evidence.input_present
                && registry_write_authority_source_evidence.source_chain_complete,
        );
    let loader_runtime_contract_source_evidence_event_id =
        event_log::record_module_service_slot_authority_input_source_evidence(
            loader_runtime_contract_source_evidence,
        );
    let health_monitor_binding_source_evidence =
        module_service_slot_authority_input_source_evidence(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[3],
            load_gate_binding,
            (
                allocation_intent_source_evidence_event_id,
                allocation_intent_source_evidence,
            ),
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[2].schema,
            Some(loader_runtime_contract_source_evidence_event_id),
            loader_runtime_contract_source_evidence.input_present
                && loader_runtime_contract_source_evidence.source_chain_complete,
        );
    let health_monitor_binding_source_evidence_event_id =
        event_log::record_module_service_slot_authority_input_source_evidence(
            health_monitor_binding_source_evidence,
        );
    let unload_cleanup_authority_source_evidence =
        module_service_slot_authority_input_source_evidence(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[4],
            load_gate_binding,
            (
                allocation_intent_source_evidence_event_id,
                allocation_intent_source_evidence,
            ),
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[3].schema,
            Some(health_monitor_binding_source_evidence_event_id),
            health_monitor_binding_source_evidence.input_present
                && health_monitor_binding_source_evidence.source_chain_complete,
        );
    let unload_cleanup_authority_source_evidence_event_id =
        event_log::record_module_service_slot_authority_input_source_evidence(
            unload_cleanup_authority_source_evidence,
        );
    let authority_input_source_evidence = [
        (
            policy_decision_source_evidence_event_id,
            policy_decision_source_evidence,
        ),
        (
            registry_write_authority_source_evidence_event_id,
            registry_write_authority_source_evidence,
        ),
        (
            loader_runtime_contract_source_evidence_event_id,
            loader_runtime_contract_source_evidence,
        ),
        (
            health_monitor_binding_source_evidence_event_id,
            health_monitor_binding_source_evidence,
        ),
        (
            unload_cleanup_authority_source_evidence_event_id,
            unload_cleanup_authority_source_evidence,
        ),
    ];
    let authority_decision_source_evidence =
        module_service_slot_allocator_authority_decision_source_evidence(
            load_gate_binding,
            (
                allocator_authority_source_evidence_event_id,
                allocator_authority_source_evidence,
            ),
            (
                allocation_intent_source_evidence_event_id,
                allocation_intent_source_evidence,
            ),
            authority_input_source_evidence,
        );
    let authority_decision_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_authority_decision_source_evidence(
            authority_decision_source_evidence,
        );
    let registry_write_commit_gate_source_evidence =
        module_service_slot_registry_write_commit_gate_source_evidence(
            load_gate_binding,
            (
                authority_decision_source_evidence_event_id,
                authority_decision_source_evidence,
            ),
            (
                registry_write_authority_source_evidence_event_id,
                registry_write_authority_source_evidence,
            ),
            registry_binding_observed_source_evidence,
            durable_audit_observed_source_evidence,
            rollback_install_observed_source_evidence,
        );
    let registry_write_commit_gate_source_evidence_event_id =
        event_log::record_module_service_slot_registry_write_commit_gate_source_evidence(
            registry_write_commit_gate_source_evidence,
        );
    let candidate = module_service_slot_allocator_snapshot(
        retained.is_some(),
        Some(allocator_runtime_observed_source_evidence),
        Some(registry_binding_observed_source_evidence),
        Some(health_state_observed_source_evidence),
        Some(unload_cleanup_observed_source_evidence),
        Some(durable_audit_observed_source_evidence),
        Some(rollback_install_observed_source_evidence),
        Some(module_loader_observed_source_evidence),
        Some((
            allocator_authority_source_evidence_event_id,
            allocator_authority_source_evidence,
        )),
        Some((
            allocation_intent_source_evidence_event_id,
            allocation_intent_source_evidence,
        )),
        Some(authority_input_source_evidence),
        Some((
            authority_decision_source_evidence_event_id,
            authority_decision_source_evidence,
        )),
        Some((
            registry_write_commit_gate_source_evidence_event_id,
            registry_write_commit_gate_source_evidence,
        )),
    );
    let evaluation = evaluate_module_service_slot_allocator_candidate(candidate);

    let item =
        |status, reason, event_id, facts| allocator_evidence(status, reason, event_id, facts);
    let source = |index: usize, present: bool| {
        vec![
            f(
                "record_schema",
                s(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[index].schema),
            ),
            f(
                "record_id",
                s(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[index].id),
            ),
            f(
                "source_method",
                s(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[index].source_method),
            ),
            f(
                "source_fact_locator",
                s(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[index].source_fact_locator),
            ),
            f("present", b(present)),
        ]
    };
    let retained_reservation = retained.map(|(_, reference)| reference);
    let retained_facts = if let Some(reference) = retained_reservation.as_ref() {
        vec![
            f(
                "record_schema",
                s("raios.module_service_slot_reservation.v0"),
            ),
            f(
                "ram_only_service_slot_id",
                s(reference.ram_only_service_slot_id.as_str()),
            ),
            f("reservation_hash", sha(reference.reservation_hash)),
            f(
                "computed_capability_grant_hash",
                sha(reference.computed_grant_hash),
            ),
            f("audit_record_hash", sha(reference.audit_record_hash)),
            f("rollback_plan_hash", sha(reference.rollback_plan_hash)),
            f(
                "pre_load_service_inventory_hash",
                sha(reference.pre_load_service_inventory_hash),
            ),
        ]
    } else {
        vec![f(
            "record_schema",
            s("raios.module_service_slot_reservation.v0"),
        )]
    };
    let authority_input = |index: usize| {
        item(
            evaluation.authority_input_statuses[index],
            evaluation.authority_input_reasons[index],
            Some(authority_input_source_evidence[index].0),
            vec![
                f(
                    "record_schema",
                    s(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[index].schema),
                ),
                f(
                    "source_method",
                    s(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[index].source_method),
                ),
                f(
                    "source_fact_locator",
                    s(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[index].source_fact_locator),
                ),
                f("present", b(candidate.authority_inputs[index].present)),
                f(
                    "source_chain_complete",
                    b(candidate.authority_inputs[index].source_chain_complete),
                ),
            ],
        )
    };
    let projection = project_allocator_denial(AllocatorProjectionInput {
        service_slot_reservation: item(
            evaluation.retained_reservation_status,
            evaluation.retained_reservation_reason,
            retained_event_id,
            retained_facts,
        ),
        service_slot_allocator_runtime: item(
            evaluation.allocator_runtime_status,
            evaluation.allocator_runtime_reason,
            Some(allocator_runtime_source_evidence_event_id),
            source(0, candidate.allocator_runtime.present),
        ),
        service_slot_registry_binding: item(
            evaluation.registry_binding_status,
            evaluation.registry_binding_reason,
            Some(registry_binding_source_evidence_event_id),
            source(1, candidate.registry_binding.present),
        ),
        service_health_state_model: item(
            evaluation.health_state_status,
            evaluation.health_state_reason,
            Some(health_state_source_evidence_event_id),
            source(2, candidate.health_state.present),
        ),
        service_unload_cleanup_plan: item(
            evaluation.unload_cleanup_status,
            evaluation.unload_cleanup_reason,
            Some(unload_cleanup_source_evidence_event_id),
            source(3, candidate.unload_cleanup.present),
        ),
        durable_audit_write: item(
            evaluation.durable_audit_status,
            evaluation.durable_audit_reason,
            Some(durable_audit_source_evidence_event_id),
            vec![f("available", b(candidate.durable_audit_write.available))],
        ),
        rollback_plan_install: item(
            evaluation.rollback_status,
            evaluation.rollback_reason,
            Some(rollback_install_source_evidence_event_id),
            vec![f("available", b(candidate.rollback_plan_install.available))],
        ),
        module_loader: item(
            evaluation.module_loader_status,
            evaluation.module_loader_reason,
            Some(module_loader_source_evidence_event_id),
            vec![f("available", b(candidate.module_loader.available))],
        ),
        service_slot_allocator_authority: item(
            evaluation.authority_status,
            evaluation.authority_reason,
            Some(allocator_authority_source_evidence_event_id),
            vec![
                f("present", b(candidate.allocator_authority.present)),
                f(
                    "source_chain_complete",
                    b(candidate.allocator_authority.source_chain_complete),
                ),
            ],
        ),
        service_slot_allocation_intent: item(
            evaluation.allocation_intent_status,
            evaluation.allocation_intent_reason,
            Some(allocation_intent_source_evidence_event_id),
            vec![
                f("present", b(candidate.allocation_intent.present)),
                f(
                    "source_chain_complete",
                    b(candidate.allocation_intent.source_chain_complete),
                ),
            ],
        ),
        policy_decision: authority_input(0),
        registry_write_authority: authority_input(1),
        loader_runtime_contract: authority_input(2),
        health_monitor_binding: authority_input(3),
        unload_cleanup_authority: authority_input(4),
        service_slot_allocator_authority_decision: item(
            evaluation.authority_decision_status,
            evaluation.authority_decision_reason,
            Some(authority_decision_source_evidence_event_id),
            vec![
                f("present", b(candidate.authority_decision.present)),
                f(
                    "source_chain_complete",
                    b(candidate.authority_decision.source_chain_complete),
                ),
            ],
        ),
        service_slot_registry_write_commit_gate: item(
            evaluation.registry_write_commit_gate_status,
            evaluation.registry_write_commit_gate_reason,
            Some(registry_write_commit_gate_source_evidence_event_id),
            vec![
                f("present", b(candidate.registry_write_commit_gate.present)),
                f(
                    "source_chain_complete",
                    b(candidate.registry_write_commit_gate.source_chain_complete),
                ),
            ],
        ),
    });
    emit_evidence_v1_response(
        "module.service_slot_allocator",
        "module.service_slot_allocator",
        None,
        V::InlineObject(vec![f("test_infrastructure", no())]),
        projection
            .evidence
            .into_iter()
            .map(ev::evidence_value)
            .collect(),
        projection.decision,
    );
}

pub(crate) fn emit_module_service_slot_allocator_selftest() {
    let cases = module_service_slot_allocator_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    let values = cases
        .iter()
        .map(|case| {
            ev::selftest_case(
                case.name,
                case.expected_status,
                case.expected_reason,
                case.actual_status,
                case.actual_reason,
                case.passed,
            )
        })
        .collect();
    emit_evidence_v1_response(
        "module.service_slot_allocator_selftest",
        "module.service_slot_allocator_selftest",
        None,
        ev::selftest_facts_value(SelftestFacts {
            case_count: cases.len() as u64,
            passed,
            safety: ev::selftest_safety_value(),
            cases: V::Array(values),
        }),
        vec![],
        ev::observed("selftest_completed"),
    );
}

fn module_service_slot_allocator_fact_source_evidence(
    source: ModuleServiceSlotAllocatorFactSource,
    retained_service_slot_reservation_event_id: Option<event_log::EventId>,
    allocator_runtime_source_evidence_event_id: Option<event_log::EventId>,
) -> event_log::ModuleServiceSlotAllocatorFactSourceEvidence {
    event_log::ModuleServiceSlotAllocatorFactSourceEvidence {
        schema: source.source_evidence_schema,
        fact_schema: source.schema,
        fact_id: source.id,
        source_method: source.source_method,
        source_fact_locator: source.source_fact_locator,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_service_slot_allocator_fact_source_evidence_recorded",
        fact_status: "missing",
        fact_reason: source.missing_reason,
        fact_present: false,
        fact_scope: "current_boot",
        fact_schema_ok: true,
        fact_provenance_ok: false,
        fact_classification: "local_only",
        retained_service_slot_reservation_present: retained_service_slot_reservation_event_id
            .is_some(),
        allocator_runtime_source_evidence_present: allocator_runtime_source_evidence_event_id
            .is_some(),
        binds_retained_service_slot_reservation: false,
        binds_allocator_runtime: false,
        retained_service_slot_reservation_event_id,
        allocator_runtime_source_evidence_event_id,
    }
}

fn module_service_slot_allocator_runtime_source_evidence(
    source: ModuleServiceSlotAllocatorFactSource,
    retained_service_slot_reservation_event_id: Option<event_log::EventId>,
    allocator_runtime_source_evidence_event_id: Option<event_log::EventId>,
) -> event_log::ModuleServiceSlotAllocatorFactSourceEvidence {
    let mut evidence = module_service_slot_allocator_fact_source_evidence(
        source,
        retained_service_slot_reservation_event_id,
        allocator_runtime_source_evidence_event_id,
    );
    if retained_service_slot_reservation_event_id.is_some() {
        evidence.fact_status = "available";
        evidence.fact_reason = "service_slot_allocator_runtime_available";
        evidence.fact_present = true;
        evidence.fact_provenance_ok = true;
        evidence.binds_retained_service_slot_reservation = true;
        evidence.binds_allocator_runtime = true;
    }
    evidence
}

fn module_service_slot_allocator_bound_fact_source_evidence(
    source: ModuleServiceSlotAllocatorFactSource,
    retained_service_slot_reservation_event_id: Option<event_log::EventId>,
    allocator_runtime_source_evidence_event_id: Option<event_log::EventId>,
    allocator_runtime_source_evidence: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    available_reason: &'static str,
) -> event_log::ModuleServiceSlotAllocatorFactSourceEvidence {
    let mut evidence = module_service_slot_allocator_fact_source_evidence(
        source,
        retained_service_slot_reservation_event_id,
        allocator_runtime_source_evidence_event_id,
    );
    if retained_service_slot_reservation_event_id.is_some()
        && allocator_runtime_source_evidence.fact_present
        && allocator_runtime_source_evidence.fact_schema_ok
        && allocator_runtime_source_evidence.fact_provenance_ok
        && allocator_runtime_source_evidence.binds_retained_service_slot_reservation
    {
        evidence.fact_status = "available";
        evidence.fact_reason = available_reason;
        evidence.fact_present = true;
        evidence.fact_provenance_ok = true;
        evidence.binds_retained_service_slot_reservation = true;
        evidence.binds_allocator_runtime = true;
    }
    evidence
}

fn module_service_slot_allocator_prerequisite_source_evidence(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
    retained_service_slot_reservation_present: bool,
    allocator_runtime_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    registry_binding_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    health_state_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    unload_cleanup_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
) -> event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence {
    event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence {
        schema: source.source_evidence_schema,
        prerequisite_schema: source.schema,
        prerequisite_id: source.id,
        source_method: source.source_method,
        source_fact_locator: source.source_fact_locator,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_service_slot_allocator_prerequisite_source_evidence_recorded",
        prerequisite_status: source.missing_status,
        prerequisite_reason: source.missing_reason,
        prerequisite_available: false,
        retained_service_slot_reservation_present,
        allocator_runtime_available: allocator_runtime_source_evidence.1.fact_present,
        registry_binding_available: registry_binding_source_evidence.1.fact_present,
        health_state_available: health_state_source_evidence.1.fact_present,
        unload_cleanup_available: unload_cleanup_source_evidence.1.fact_present,
        allocator_runtime_source_evidence_event_id: Some(allocator_runtime_source_evidence.0),
        registry_binding_source_evidence_event_id: Some(registry_binding_source_evidence.0),
        health_state_source_evidence_event_id: Some(health_state_source_evidence.0),
        unload_cleanup_source_evidence_event_id: Some(unload_cleanup_source_evidence.0),
    }
}

fn module_service_slot_allocator_bound_prerequisite_source_evidence(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
    retained_service_slot_reservation_present: bool,
    allocator_runtime_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    registry_binding_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    health_state_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    unload_cleanup_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    available_reason: &'static str,
) -> event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence {
    let mut evidence = module_service_slot_allocator_prerequisite_source_evidence(
        source,
        retained_service_slot_reservation_present,
        allocator_runtime_source_evidence,
        registry_binding_source_evidence,
        health_state_source_evidence,
        unload_cleanup_source_evidence,
    );
    if retained_service_slot_reservation_present
        && module_service_slot_allocator_fact_source_evidence_available(
            allocator_runtime_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            registry_binding_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            health_state_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            unload_cleanup_source_evidence.1,
        )
    {
        evidence.prerequisite_status = "available";
        evidence.prerequisite_reason = available_reason;
        evidence.prerequisite_available = true;
    }
    evidence
}

fn module_service_slot_allocator_fact_source_evidence_available(
    evidence: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
) -> bool {
    evidence.fact_present
        && evidence.fact_schema_ok
        && evidence.fact_provenance_ok
        && evidence.binds_retained_service_slot_reservation
        && evidence.binds_allocator_runtime
}

fn module_service_slot_allocator_module_loader_prerequisite_source_evidence(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
    retained_service_slot_reservation_present: bool,
    allocator_runtime_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    registry_binding_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    health_state_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    unload_cleanup_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    durable_audit_source_evidence: event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    rollback_install_source_evidence: event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
) -> event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence {
    let mut evidence = module_service_slot_allocator_prerequisite_source_evidence(
        source,
        retained_service_slot_reservation_present,
        allocator_runtime_source_evidence,
        registry_binding_source_evidence,
        health_state_source_evidence,
        unload_cleanup_source_evidence,
    );
    if retained_service_slot_reservation_present
        && module_service_slot_allocator_fact_source_evidence_available(
            allocator_runtime_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            registry_binding_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            health_state_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            unload_cleanup_source_evidence.1,
        )
        && module_service_slot_allocator_prerequisite_source_evidence_available(
            durable_audit_source_evidence,
        )
        && module_service_slot_allocator_prerequisite_source_evidence_available(
            rollback_install_source_evidence,
        )
    {
        evidence.prerequisite_status = "available";
        evidence.prerequisite_reason = "module_loader_boundary_available_non_authorizing";
        evidence.prerequisite_available = true;
    }
    evidence
}

fn module_service_slot_allocator_prerequisite_source_evidence_available(
    evidence: event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
) -> bool {
    evidence.prerequisite_available
}

fn module_service_slot_allocator_authority_source_evidence(
    retained_service_slot_reservation_present: bool,
    allocator_runtime_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    registry_binding_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    health_state_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    unload_cleanup_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    durable_audit_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    ),
    rollback_install_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    ),
    module_loader_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    ),
) -> event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence {
    let source_chain_complete = retained_service_slot_reservation_present
        && module_service_slot_allocator_fact_source_evidence_available(
            allocator_runtime_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            registry_binding_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            health_state_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            unload_cleanup_source_evidence.1,
        )
        && module_service_slot_allocator_prerequisite_source_evidence_available(
            durable_audit_source_evidence.1,
        )
        && module_service_slot_allocator_prerequisite_source_evidence_available(
            rollback_install_source_evidence.1,
        )
        && module_service_slot_allocator_prerequisite_source_evidence_available(
            module_loader_source_evidence.1,
        );
    event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence {
        schema: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_SCHEMA,
        authority_schema: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SCHEMA,
        authority_id: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_ID,
        source_method: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_METHOD,
        source_fact_locator: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_FACT_LOCATOR,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_service_slot_allocator_authority_source_evidence_recorded",
        authority_status: if source_chain_complete {
            "defined_non_authorizing"
        } else {
            "missing"
        },
        authority_reason: if source_chain_complete {
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON
        } else {
            "service_slot_allocator_authority_source_chain_incomplete"
        },
        authority_present: source_chain_complete,
        authority_scope: "current_boot",
        authority_schema_ok: true,
        authority_provenance_ok: source_chain_complete,
        authority_classification: "local_only",
        retained_service_slot_reservation_present,
        allocator_runtime_available: allocator_runtime_source_evidence.1.fact_present,
        registry_binding_available: registry_binding_source_evidence.1.fact_present,
        health_state_available: health_state_source_evidence.1.fact_present,
        unload_cleanup_available: unload_cleanup_source_evidence.1.fact_present,
        durable_audit_write_available: durable_audit_source_evidence.1.prerequisite_available,
        rollback_plan_install_available: rollback_install_source_evidence.1.prerequisite_available,
        module_loader_available: module_loader_source_evidence.1.prerequisite_available,
        source_chain_complete,
        allocator_runtime_source_evidence_event_id: Some(allocator_runtime_source_evidence.0),
        registry_binding_source_evidence_event_id: Some(registry_binding_source_evidence.0),
        health_state_source_evidence_event_id: Some(health_state_source_evidence.0),
        unload_cleanup_source_evidence_event_id: Some(unload_cleanup_source_evidence.0),
        durable_audit_source_evidence_event_id: Some(durable_audit_source_evidence.0),
        rollback_install_source_evidence_event_id: Some(rollback_install_source_evidence.0),
        module_loader_source_evidence_event_id: Some(module_loader_source_evidence.0),
    }
}

fn module_service_slot_allocation_intent_source_evidence(
    binding: event_log::ModuleLoadGateBinding,
    retained_service_slot_reservation_present: bool,
    allocator_authority_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    ),
) -> event_log::ModuleServiceSlotAllocationIntentSourceEvidence {
    let retained_module_evidence_present =
        module_service_slot_allocation_intent_retained_module_evidence_present(binding);
    let source_chain_complete = retained_module_evidence_present
        && retained_service_slot_reservation_present
        && allocator_authority_source_evidence.1.authority_present
        && allocator_authority_source_evidence.1.source_chain_complete;
    event_log::ModuleServiceSlotAllocationIntentSourceEvidence {
        schema: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_SCHEMA,
        intent_schema: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SCHEMA,
        intent_id: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_ID,
        source_method: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_METHOD,
        source_fact_locator: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_FACT_LOCATOR,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_service_slot_allocation_intent_source_evidence_recorded",
        intent_status: if source_chain_complete {
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_STATUS
        } else {
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS
        },
        intent_reason: if source_chain_complete {
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_REASON
        } else {
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_CHAIN_INCOMPLETE_REASON
        },
        intent_present: source_chain_complete,
        intent_scope: "current_boot",
        intent_schema_ok: true,
        intent_provenance_ok: source_chain_complete,
        intent_classification: "local_only",
        requested_capability: "cap.module.load_ephemeral",
        load_mode: "ram_only",
        target: "live_service_graph",
        retained_module_evidence_present,
        retained_service_slot_reservation_present,
        allocator_authority_present: allocator_authority_source_evidence.1.authority_present,
        source_chain_complete,
        manifest_reference_event_id: binding.manifest_reference_event_id,
        artifact_reference_event_id: binding.artifact_reference_event_id,
        vm_report_reference_event_id: binding.vm_report_reference_event_id,
        local_attestation_reference_event_id: binding.attestation_reference_event_id,
        local_approval_reference_event_id: binding.approval_reference_event_id,
        computed_grant_reference_event_id: binding.retained_reference_event_id,
        audit_rollback_reference_event_id: binding.audit_rollback_reference_event_id,
        service_slot_reservation_event_id: binding.service_slot_reservation_event_id,
        allocator_authority_source_evidence_event_id: Some(allocator_authority_source_evidence.0),
        ram_only_service_slot_id: binding
            .service_slot_reservation
            .map(|reservation| reservation.ram_only_service_slot_id),
    }
}

fn module_service_slot_allocation_intent_retained_module_evidence_present(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.manifest_reference_status,
        "retained_hash_reference_only",
    ) && method_eq(
        binding.artifact_reference_status,
        "retained_hash_reference_only",
    ) && method_eq(
        binding.vm_report_reference_status,
        "retained_hash_reference_only",
    ) && method_eq(
        binding.attestation_reference_status,
        "retained_hash_reference_only",
    ) && method_eq(
        binding.approval_reference_status,
        "retained_hash_reference_only",
    ) && binding.retained_reference_event_id.is_some()
        && method_eq(
            binding.audit_rollback_reference_status,
            "retained_hash_reference_only",
        )
}

fn module_service_slot_authority_input_source_evidence(
    source: ModuleServiceSlotAuthorityInputSpec,
    binding: event_log::ModuleLoadGateBinding,
    allocation_intent_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
    ),
    dependency_schema: &'static str,
    dependency_source_evidence_event_id: Option<event_log::EventId>,
    dependency_present: bool,
) -> event_log::ModuleServiceSlotAuthorityInputSourceEvidence {
    let source_chain_complete = allocation_intent_source_evidence.1.source_chain_complete
        && allocation_intent_source_evidence.1.intent_present
        && dependency_present
        && dependency_source_evidence_event_id.is_some();
    event_log::ModuleServiceSlotAuthorityInputSourceEvidence {
        schema: source.source_evidence_schema,
        input_schema: source.schema,
        input_id: source.id,
        input_name: source.name,
        source_method: source.source_method,
        source_fact_locator: source.source_fact_locator,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_service_slot_authority_input_source_evidence_recorded",
        input_status: if source_chain_complete {
            source.available_status
        } else {
            source.missing_status
        },
        input_reason: if source_chain_complete {
            source.available_reason
        } else {
            source.source_chain_incomplete_reason
        },
        input_present: source_chain_complete,
        input_scope: "current_boot",
        input_schema_ok: true,
        input_provenance_ok: source_chain_complete,
        input_classification: "local_only",
        dependency_schema,
        dependency_source_evidence_event_id,
        dependency_present,
        requested_capability: "cap.module.load_ephemeral",
        load_mode: "ram_only",
        target: "live_service_graph",
        retained_module_evidence_present: allocation_intent_source_evidence
            .1
            .retained_module_evidence_present,
        retained_service_slot_reservation_present: allocation_intent_source_evidence
            .1
            .retained_service_slot_reservation_present,
        allocator_authority_present: allocation_intent_source_evidence
            .1
            .allocator_authority_present,
        allocation_intent_source_evidence_event_id: Some(allocation_intent_source_evidence.0),
        source_chain_complete,
        service_slot_reservation_event_id: binding.service_slot_reservation_event_id,
        allocator_authority_source_evidence_event_id: allocation_intent_source_evidence
            .1
            .allocator_authority_source_evidence_event_id,
        ram_only_service_slot_id: binding
            .service_slot_reservation
            .map(|reservation| reservation.ram_only_service_slot_id),
    }
}

fn module_service_slot_allocator_authority_decision_source_evidence(
    binding: event_log::ModuleLoadGateBinding,
    allocator_authority_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    ),
    allocation_intent_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
    ),
    authority_input_source_evidence: [(
        event_log::EventId,
        event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
    ); MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
) -> event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence {
    let mut authority_inputs_complete = true;
    let mut authority_input_event_ids = [None; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT];
    let mut authority_input_present = [false; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT];
    let mut idx = 0usize;
    while idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        authority_input_event_ids[idx] = Some(authority_input_source_evidence[idx].0);
        authority_input_present[idx] = authority_input_source_evidence[idx].1.input_present
            && authority_input_source_evidence[idx].1.source_chain_complete;
        authority_inputs_complete = authority_inputs_complete && authority_input_present[idx];
        idx += 1;
    }
    let source_chain_complete = allocator_authority_source_evidence.1.authority_present
        && allocator_authority_source_evidence.1.source_chain_complete
        && allocation_intent_source_evidence.1.intent_present
        && allocation_intent_source_evidence.1.source_chain_complete
        && authority_inputs_complete;
    event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence {
        schema: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_SCHEMA,
        decision_schema: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SCHEMA,
        decision_id: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_ID,
        source_method: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_METHOD,
        source_fact_locator: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_FACT_LOCATOR,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason:
            "module_service_slot_allocator_authority_decision_source_evidence_recorded",
        decision_status: if source_chain_complete {
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_STATUS
        } else {
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS
        },
        decision_reason: if source_chain_complete {
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_REASON
        } else {
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_CHAIN_INCOMPLETE_REASON
        },
        decision_present: source_chain_complete,
        decision_scope: "current_boot",
        decision_schema_ok: true,
        decision_provenance_ok: source_chain_complete,
        decision_classification: "local_only",
        requested_capability: "cap.module.load_ephemeral",
        load_mode: "ram_only",
        target: "live_service_graph",
        allocator_authority_present: allocator_authority_source_evidence.1.authority_present,
        allocation_intent_present: allocation_intent_source_evidence.1.intent_present,
        authority_inputs_complete,
        source_chain_complete,
        allocator_authority_source_evidence_event_id: Some(allocator_authority_source_evidence.0),
        allocation_intent_source_evidence_event_id: Some(allocation_intent_source_evidence.0),
        authority_input_source_evidence_event_ids: authority_input_event_ids,
        authority_input_present,
        retained_module_evidence_present: allocation_intent_source_evidence
            .1
            .retained_module_evidence_present,
        retained_service_slot_reservation_present: allocation_intent_source_evidence
            .1
            .retained_service_slot_reservation_present,
        service_slot_reservation_event_id: binding.service_slot_reservation_event_id,
        ram_only_service_slot_id: binding
            .service_slot_reservation
            .map(|reservation| reservation.ram_only_service_slot_id),
    }
}

fn module_service_slot_registry_write_commit_gate_source_evidence(
    binding: event_log::ModuleLoadGateBinding,
    authority_decision_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
    ),
    registry_write_authority_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
    ),
    registry_binding_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    durable_audit_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    ),
    rollback_install_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    ),
) -> event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence {
    let retained_service_slot_reservation_present = binding.service_slot_reservation.is_some()
        && binding.service_slot_reservation_event_id.is_some();
    let authority_decision_present = authority_decision_source_evidence.1.decision_present
        && authority_decision_source_evidence.1.source_chain_complete;
    let registry_write_authority_present = registry_write_authority_source_evidence.1.input_present
        && registry_write_authority_source_evidence
            .1
            .source_chain_complete;
    let registry_binding_available = registry_binding_source_evidence.1.fact_present
        && registry_binding_source_evidence.1.fact_schema_ok
        && registry_binding_source_evidence.1.fact_provenance_ok
        && registry_binding_source_evidence
            .1
            .binds_retained_service_slot_reservation
        && registry_binding_source_evidence.1.binds_allocator_runtime;
    let durable_audit_write_available = durable_audit_source_evidence.1.prerequisite_available;
    let rollback_plan_install_available = rollback_install_source_evidence.1.prerequisite_available;
    let source_chain_complete = authority_decision_present
        && registry_write_authority_present
        && registry_binding_available
        && durable_audit_write_available
        && rollback_plan_install_available
        && retained_service_slot_reservation_present;
    event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence {
        schema: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_SCHEMA,
        gate_schema: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SCHEMA,
        gate_id: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_ID,
        source_method: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_METHOD,
        source_fact_locator: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_FACT_LOCATOR,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_service_slot_registry_write_commit_gate_source_evidence_recorded",
        gate_status: if source_chain_complete {
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_STATUS
        } else {
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS
        },
        gate_reason: if source_chain_complete {
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_REASON
        } else {
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_CHAIN_INCOMPLETE_REASON
        },
        gate_present: source_chain_complete,
        gate_scope: "current_boot",
        gate_schema_ok: true,
        gate_provenance_ok: source_chain_complete,
        gate_classification: "local_only",
        requested_capability: "cap.module.load_ephemeral",
        load_mode: "ram_only",
        target: "live_service_graph",
        authority_decision_present,
        registry_write_authority_present,
        registry_binding_available,
        durable_audit_write_available,
        rollback_plan_install_available,
        retained_service_slot_reservation_present,
        source_chain_complete,
        authority_decision_source_evidence_event_id: Some(authority_decision_source_evidence.0),
        registry_write_authority_source_evidence_event_id: Some(
            registry_write_authority_source_evidence.0,
        ),
        registry_binding_source_evidence_event_id: Some(registry_binding_source_evidence.0),
        durable_audit_source_evidence_event_id: Some(durable_audit_source_evidence.0),
        rollback_install_source_evidence_event_id: Some(rollback_install_source_evidence.0),
        service_slot_reservation_event_id: binding.service_slot_reservation_event_id,
        ram_only_service_slot_id: binding
            .service_slot_reservation
            .map(|reservation| reservation.ram_only_service_slot_id),
        authorizes_registry_write: false,
        mutates_service_registry: false,
        writes_durable_audit_state: false,
        installs_rollback_state: false,
        allocates_service_slot: false,
        loads_artifact: false,
    }
}

fn module_service_slot_allocator_snapshot(
    retained_reservation_present: bool,
    allocator_runtime_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    )>,
    registry_binding_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    )>,
    health_state_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    )>,
    unload_cleanup_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    )>,
    durable_audit_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    )>,
    rollback_install_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    )>,
    module_loader_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    )>,
    allocator_authority_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    )>,
    allocation_intent_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
    )>,
    authority_input_source_evidence: Option<
        [(
            event_log::EventId,
            event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
        ); MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
    >,
    authority_decision_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
    )>,
    registry_write_commit_gate_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence,
    )>,
) -> ModuleServiceSlotAllocatorCandidate {
    ModuleServiceSlotAllocatorCandidate {
        retained_reservation_present,
        allocator_runtime: module_service_slot_allocator_fact_from_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0],
            allocator_runtime_source_evidence,
        ),
        registry_binding: module_service_slot_allocator_fact_from_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1],
            registry_binding_source_evidence,
        ),
        health_state: module_service_slot_allocator_fact_from_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2],
            health_state_source_evidence,
        ),
        unload_cleanup: module_service_slot_allocator_fact_from_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3],
            unload_cleanup_source_evidence,
        ),
        durable_audit_write: module_service_slot_allocator_prerequisite_from_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0],
            durable_audit_source_evidence,
        ),
        rollback_plan_install: module_service_slot_allocator_prerequisite_from_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1],
            rollback_install_source_evidence,
        ),
        module_loader: module_service_slot_allocator_prerequisite_from_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2],
            module_loader_source_evidence,
        ),
        allocator_authority: module_service_slot_allocator_authority_from_source_evidence(
            allocator_authority_source_evidence,
        ),
        allocation_intent: module_service_slot_allocation_intent_from_source_evidence(
            allocation_intent_source_evidence,
        ),
        authority_inputs: module_service_slot_authority_inputs_from_source_evidence(
            authority_input_source_evidence,
        ),
        authority_decision: module_service_slot_allocator_authority_decision_from_source_evidence(
            authority_decision_source_evidence,
        ),
        registry_write_commit_gate:
            module_service_slot_registry_write_commit_gate_from_source_evidence(
                registry_write_commit_gate_source_evidence,
            ),
    }
}

fn module_service_slot_allocator_empty_snapshot(
    retained_reservation_present: bool,
) -> ModuleServiceSlotAllocatorCandidate {
    module_service_slot_allocator_snapshot(
        retained_reservation_present,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
}

fn module_service_slot_allocator_authority_from_source_evidence(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    )>,
) -> ModuleServiceSlotAllocatorAuthority {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleServiceSlotAllocatorAuthority {
            present: evidence.authority_present,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.authority_present {
                "observed_current_boot_defined"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.authority_status,
            source_evidence_reason: evidence.authority_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
            source_chain_complete: evidence.source_chain_complete,
        };
    }
    module_service_slot_allocator_missing_authority()
}

fn module_service_slot_allocator_missing_authority() -> ModuleServiceSlotAllocatorAuthority {
    ModuleServiceSlotAllocatorAuthority {
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: "missing",
        source_evidence_reason:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_METHOD,
        source_evidence_fact_locator: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_FACT_LOCATOR,
        source_chain_complete: false,
    }
}

fn module_service_slot_allocation_intent_from_source_evidence(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
    )>,
) -> ModuleServiceSlotAllocationIntent {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleServiceSlotAllocationIntent {
            present: evidence.intent_present,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.intent_present {
                "observed_current_boot_defined"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.intent_status,
            source_evidence_reason: evidence.intent_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
            source_chain_complete: evidence.source_chain_complete,
        };
    }
    module_service_slot_allocation_intent_missing()
}

fn module_service_slot_allocation_intent_missing() -> ModuleServiceSlotAllocationIntent {
    ModuleServiceSlotAllocationIntent {
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
        source_evidence_reason:
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_METHOD,
        source_evidence_fact_locator: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_FACT_LOCATOR,
        source_chain_complete: false,
    }
}

fn module_service_slot_authority_inputs_from_source_evidence(
    source_evidence: Option<
        [(
            event_log::EventId,
            event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
        ); MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
    >,
) -> [ModuleServiceSlotAuthorityInput; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT] {
    if let Some(source_evidence) = source_evidence {
        return [
            module_service_slot_authority_input_from_source_evidence(
                MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0],
                Some(source_evidence[0]),
            ),
            module_service_slot_authority_input_from_source_evidence(
                MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[1],
                Some(source_evidence[1]),
            ),
            module_service_slot_authority_input_from_source_evidence(
                MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[2],
                Some(source_evidence[2]),
            ),
            module_service_slot_authority_input_from_source_evidence(
                MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[3],
                Some(source_evidence[3]),
            ),
            module_service_slot_authority_input_from_source_evidence(
                MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[4],
                Some(source_evidence[4]),
            ),
        ];
    }
    module_service_slot_authority_inputs_missing()
}

fn module_service_slot_authority_input_from_source_evidence(
    source: ModuleServiceSlotAuthorityInputSpec,
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
    )>,
) -> ModuleServiceSlotAuthorityInput {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleServiceSlotAuthorityInput {
            spec: source,
            present: evidence.input_present,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.input_present {
                "observed_current_boot_defined"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.input_status,
            source_evidence_reason: evidence.input_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
            dependency_source_evidence_event_id: evidence.dependency_source_evidence_event_id,
            source_chain_complete: evidence.source_chain_complete,
        };
    }
    module_service_slot_authority_input_missing(source)
}

fn module_service_slot_authority_inputs_missing(
) -> [ModuleServiceSlotAuthorityInput; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT] {
    [
        module_service_slot_authority_input_missing(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0]),
        module_service_slot_authority_input_missing(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[1]),
        module_service_slot_authority_input_missing(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[2]),
        module_service_slot_authority_input_missing(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[3]),
        module_service_slot_authority_input_missing(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[4]),
    ]
}

fn module_service_slot_authority_input_missing(
    source: ModuleServiceSlotAuthorityInputSpec,
) -> ModuleServiceSlotAuthorityInput {
    ModuleServiceSlotAuthorityInput {
        spec: source,
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema: source.source_evidence_schema,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: source.missing_status,
        source_evidence_reason: source.source_evidence_missing_reason,
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
        dependency_source_evidence_event_id: None,
        source_chain_complete: false,
    }
}

fn module_service_slot_allocator_authority_decision_from_source_evidence(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
    )>,
) -> ModuleServiceSlotAllocatorAuthorityDecision {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleServiceSlotAllocatorAuthorityDecision {
            present: evidence.decision_present,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.decision_present {
                "observed_current_boot_defined"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.decision_status,
            source_evidence_reason: evidence.decision_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
            source_chain_complete: evidence.source_chain_complete,
            input_chain_complete: evidence.authority_inputs_complete,
        };
    }
    module_service_slot_allocator_authority_decision_missing()
}

fn module_service_slot_allocator_authority_decision_missing(
) -> ModuleServiceSlotAllocatorAuthorityDecision {
    ModuleServiceSlotAllocatorAuthorityDecision {
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
        source_evidence_reason:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_FACT_LOCATOR,
        source_chain_complete: false,
        input_chain_complete: false,
    }
}

fn module_service_slot_registry_write_commit_gate_from_source_evidence(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence,
    )>,
) -> ModuleServiceSlotRegistryWriteCommitGate {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleServiceSlotRegistryWriteCommitGate {
            present: evidence.gate_present,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.gate_present {
                "observed_current_boot_defined"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.gate_status,
            source_evidence_reason: evidence.gate_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
            source_chain_complete: evidence.source_chain_complete,
            authority_decision_present: evidence.authority_decision_present,
            registry_write_authority_present: evidence.registry_write_authority_present,
            registry_binding_available: evidence.registry_binding_available,
            durable_audit_write_available: evidence.durable_audit_write_available,
            rollback_plan_install_available: evidence.rollback_plan_install_available,
            retained_service_slot_reservation_present: evidence
                .retained_service_slot_reservation_present,
        };
    }
    module_service_slot_registry_write_commit_gate_missing()
}

fn module_service_slot_registry_write_commit_gate_missing(
) -> ModuleServiceSlotRegistryWriteCommitGate {
    ModuleServiceSlotRegistryWriteCommitGate {
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
        source_evidence_reason:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_FACT_LOCATOR,
        source_chain_complete: false,
        authority_decision_present: false,
        registry_write_authority_present: false,
        registry_binding_available: false,
        durable_audit_write_available: false,
        rollback_plan_install_available: false,
        retained_service_slot_reservation_present: false,
    }
}

fn module_service_slot_allocator_fact_from_source_evidence(
    source: ModuleServiceSlotAllocatorFactSource,
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    )>,
) -> ModuleServiceSlotAllocatorFact {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleServiceSlotAllocatorFact {
            present: evidence.fact_present,
            schema_ok: evidence.fact_schema_ok,
            scope: evidence.fact_scope,
            provenance_ok: evidence.fact_provenance_ok,
            classification: evidence.fact_classification,
            binds_retained_reservation: evidence.binds_retained_service_slot_reservation,
            binds_allocator_runtime: evidence.binds_allocator_runtime,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.fact_present {
                "observed_current_boot_available"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.fact_status,
            source_evidence_reason: evidence.fact_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
        };
    }
    module_service_slot_allocator_missing_fact(source)
}

fn module_service_slot_allocator_missing_fact(
    source: ModuleServiceSlotAllocatorFactSource,
) -> ModuleServiceSlotAllocatorFact {
    ModuleServiceSlotAllocatorFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_retained_reservation: false,
        binds_allocator_runtime: false,
        source_evidence_event_id: None,
        source_evidence_schema: source.source_evidence_schema,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: "missing",
        source_evidence_reason: source.source_evidence_missing_reason,
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
    }
}

fn module_service_slot_allocator_available_fact(
    source: ModuleServiceSlotAllocatorFactSource,
) -> ModuleServiceSlotAllocatorFact {
    ModuleServiceSlotAllocatorFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_retained_reservation: true,
        binds_allocator_runtime: true,
        source_evidence_event_id: None,
        source_evidence_schema: source.source_evidence_schema,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: "available",
        source_evidence_reason: "service_slot_allocator_fact_available",
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
    }
}

fn module_service_slot_allocator_prerequisite_from_source_evidence(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    )>,
) -> ModuleServiceSlotAllocatorPrerequisite {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleServiceSlotAllocatorPrerequisite {
            available: evidence.prerequisite_available,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.prerequisite_available {
                "observed_current_boot_available"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.prerequisite_status,
            source_evidence_reason: evidence.prerequisite_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
        };
    }
    module_service_slot_allocator_missing_prerequisite(source)
}

fn module_service_slot_allocator_missing_prerequisite(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
) -> ModuleServiceSlotAllocatorPrerequisite {
    ModuleServiceSlotAllocatorPrerequisite {
        available: false,
        source_evidence_event_id: None,
        source_evidence_schema: source.source_evidence_schema,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: source.missing_status,
        source_evidence_reason: source.source_evidence_missing_reason,
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
    }
}

fn module_service_slot_allocator_available_prerequisite(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
) -> ModuleServiceSlotAllocatorPrerequisite {
    ModuleServiceSlotAllocatorPrerequisite {
        available: true,
        source_evidence_event_id: None,
        source_evidence_schema: source.source_evidence_schema,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: "available",
        source_evidence_reason: "service_slot_allocator_prerequisite_available",
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
    }
}

fn module_service_slot_allocator_available_authority() -> ModuleServiceSlotAllocatorAuthority {
    ModuleServiceSlotAllocatorAuthority {
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: "defined_non_authorizing",
        source_evidence_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_METHOD,
        source_evidence_fact_locator: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_FACT_LOCATOR,
        source_chain_complete: true,
    }
}

fn module_service_slot_allocation_intent_available() -> ModuleServiceSlotAllocationIntent {
    ModuleServiceSlotAllocationIntent {
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_STATUS,
        source_evidence_reason: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_METHOD,
        source_evidence_fact_locator: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_FACT_LOCATOR,
        source_chain_complete: true,
    }
}

fn module_service_slot_authority_inputs_available(
) -> [ModuleServiceSlotAuthorityInput; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT] {
    [
        module_service_slot_authority_input_available(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0],
        ),
        module_service_slot_authority_input_available(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[1],
        ),
        module_service_slot_authority_input_available(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[2],
        ),
        module_service_slot_authority_input_available(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[3],
        ),
        module_service_slot_authority_input_available(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[4],
        ),
    ]
}

fn module_service_slot_authority_input_available(
    source: ModuleServiceSlotAuthorityInputSpec,
) -> ModuleServiceSlotAuthorityInput {
    ModuleServiceSlotAuthorityInput {
        spec: source,
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema: source.source_evidence_schema,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: source.available_status,
        source_evidence_reason: source.available_reason,
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
        dependency_source_evidence_event_id: None,
        source_chain_complete: true,
    }
}

fn module_service_slot_allocator_authority_decision_available(
) -> ModuleServiceSlotAllocatorAuthorityDecision {
    ModuleServiceSlotAllocatorAuthorityDecision {
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_STATUS,
        source_evidence_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_FACT_LOCATOR,
        source_chain_complete: true,
        input_chain_complete: true,
    }
}

fn module_service_slot_registry_write_commit_gate_available(
) -> ModuleServiceSlotRegistryWriteCommitGate {
    ModuleServiceSlotRegistryWriteCommitGate {
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_STATUS,
        source_evidence_reason: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_FACT_LOCATOR,
        source_chain_complete: true,
        authority_decision_present: true,
        registry_write_authority_present: true,
        registry_binding_available: true,
        durable_audit_write_available: true,
        rollback_plan_install_available: true,
        retained_service_slot_reservation_present: true,
    }
}

fn evaluate_module_service_slot_allocator_candidate(
    candidate: ModuleServiceSlotAllocatorCandidate,
) -> ModuleServiceSlotAllocatorEvaluation {
    let retained_reservation_status = if candidate.retained_reservation_present {
        "available"
    } else {
        "missing"
    };
    let retained_reservation_reason = if candidate.retained_reservation_present {
        "retained_service_slot_reservation_available"
    } else {
        "retained_service_slot_reservation_missing"
    };

    let (allocator_runtime_status, allocator_runtime_reason) =
        evaluate_module_service_slot_allocator_fact(
            candidate.allocator_runtime,
            "service_slot_allocator_scope_must_be_current_boot",
            "service_slot_allocator_schema_mismatch",
            "service_slot_allocator_runtime_missing",
            "service_slot_allocator_provenance_missing",
            "service_slot_allocator_retained_reservation_binding_missing",
            None,
            "service_slot_allocator_runtime_available",
        );
    let (registry_binding_status, registry_binding_reason) =
        evaluate_module_service_slot_allocator_fact(
            candidate.registry_binding,
            "service_slot_registry_binding_scope_must_be_current_boot",
            "service_slot_registry_binding_schema_mismatch",
            "service_slot_registry_binding_missing",
            "service_slot_registry_binding_provenance_missing",
            "service_slot_registry_retained_reservation_binding_missing",
            Some("service_slot_registry_allocator_runtime_binding_missing"),
            "service_slot_registry_binding_available",
        );
    let (health_state_status, health_state_reason) = evaluate_module_service_slot_allocator_fact(
        candidate.health_state,
        "service_health_state_scope_must_be_current_boot",
        "service_health_state_schema_mismatch",
        "service_health_state_model_missing",
        "service_health_state_provenance_missing",
        "service_health_state_retained_reservation_binding_missing",
        Some("service_health_state_allocator_runtime_binding_missing"),
        "service_health_state_model_available",
    );
    let (unload_cleanup_status, unload_cleanup_reason) =
        evaluate_module_service_slot_allocator_fact(
            candidate.unload_cleanup,
            "service_unload_cleanup_scope_must_be_current_boot",
            "service_unload_cleanup_schema_mismatch",
            "service_unload_cleanup_plan_missing",
            "service_unload_cleanup_provenance_missing",
            "service_unload_cleanup_retained_reservation_binding_missing",
            Some("service_unload_cleanup_allocator_runtime_binding_missing"),
            "service_unload_cleanup_plan_available",
        );

    let durable_audit_status = if candidate.durable_audit_write.available {
        "available"
    } else {
        "missing"
    };
    let durable_audit_reason = if candidate.durable_audit_write.available {
        "durable_audit_write_available"
    } else {
        "durable_audit_write_missing"
    };
    let rollback_status = if candidate.rollback_plan_install.available {
        "available"
    } else {
        "missing"
    };
    let rollback_reason = if candidate.rollback_plan_install.available {
        "rollback_plan_install_available"
    } else {
        "rollback_install_missing"
    };
    let module_loader_status = if candidate.module_loader.available {
        "available"
    } else {
        "unavailable"
    };
    let module_loader_reason = if candidate.module_loader.available {
        "module_loader_boundary_available_non_authorizing"
    } else {
        "module_loader_unimplemented"
    };
    let authority_status = if candidate.allocator_authority.present
        && candidate.allocator_authority.source_chain_complete
    {
        "defined_non_authorizing"
    } else {
        "missing"
    };
    let authority_reason = if candidate.allocator_authority.present
        && candidate.allocator_authority.source_chain_complete
    {
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON
    } else {
        "service_slot_allocator_authority_source_chain_incomplete"
    };
    let allocation_intent_status = if candidate.allocation_intent.present
        && candidate.allocation_intent.source_chain_complete
    {
        MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_STATUS
    } else {
        MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS
    };
    let allocation_intent_reason = if candidate.allocation_intent.present
        && candidate.allocation_intent.source_chain_complete
    {
        MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_REASON
    } else {
        MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_CHAIN_INCOMPLETE_REASON
    };
    let authority_input_statuses = [
        module_service_slot_authority_input_status(candidate.authority_inputs[0]),
        module_service_slot_authority_input_status(candidate.authority_inputs[1]),
        module_service_slot_authority_input_status(candidate.authority_inputs[2]),
        module_service_slot_authority_input_status(candidate.authority_inputs[3]),
        module_service_slot_authority_input_status(candidate.authority_inputs[4]),
    ];
    let authority_input_reasons = [
        module_service_slot_authority_input_reason(candidate.authority_inputs[0]),
        module_service_slot_authority_input_reason(candidate.authority_inputs[1]),
        module_service_slot_authority_input_reason(candidate.authority_inputs[2]),
        module_service_slot_authority_input_reason(candidate.authority_inputs[3]),
        module_service_slot_authority_input_reason(candidate.authority_inputs[4]),
    ];
    let authority_decision_status = if candidate.authority_decision.present
        && candidate.authority_decision.source_chain_complete
        && candidate.authority_decision.input_chain_complete
    {
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_STATUS
    } else {
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS
    };
    let authority_decision_reason = if candidate.authority_decision.present
        && candidate.authority_decision.source_chain_complete
        && candidate.authority_decision.input_chain_complete
    {
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_REASON
    } else {
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_CHAIN_INCOMPLETE_REASON
    };
    let registry_write_commit_gate_status = if candidate.registry_write_commit_gate.present
        && candidate.registry_write_commit_gate.source_chain_complete
    {
        MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_STATUS
    } else {
        MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS
    };
    let registry_write_commit_gate_reason = if candidate.registry_write_commit_gate.present
        && candidate.registry_write_commit_gate.source_chain_complete
    {
        MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_REASON
    } else {
        MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_CHAIN_INCOMPLETE_REASON
    };
    let mut authority_inputs_complete = true;
    let mut first_authority_input_reason =
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0].source_chain_incomplete_reason;
    let mut authority_input_idx = 0usize;
    while authority_input_idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        if !candidate.authority_inputs[authority_input_idx].present
            || !candidate.authority_inputs[authority_input_idx].source_chain_complete
        {
            authority_inputs_complete = false;
            first_authority_input_reason = authority_input_reasons[authority_input_idx];
            break;
        }
        authority_input_idx += 1;
    }

    let (status, reason) = if !candidate.retained_reservation_present {
        ("missing", retained_reservation_reason)
    } else if method_eq(allocator_runtime_status, "rejected") {
        ("rejected", allocator_runtime_reason)
    } else if method_eq(allocator_runtime_status, "missing") {
        ("missing", allocator_runtime_reason)
    } else if method_eq(registry_binding_status, "rejected") {
        ("rejected", registry_binding_reason)
    } else if method_eq(registry_binding_status, "missing") {
        ("missing", registry_binding_reason)
    } else if method_eq(health_state_status, "rejected") {
        ("rejected", health_state_reason)
    } else if method_eq(health_state_status, "missing") {
        ("missing", health_state_reason)
    } else if method_eq(unload_cleanup_status, "rejected") {
        ("rejected", unload_cleanup_reason)
    } else if method_eq(unload_cleanup_status, "missing") {
        ("missing", unload_cleanup_reason)
    } else if !candidate.durable_audit_write.available {
        ("denied_missing_durable_audit_write", durable_audit_reason)
    } else if !candidate.rollback_plan_install.available {
        ("denied_missing_rollback_install", rollback_reason)
    } else if !candidate.module_loader.available {
        ("denied_loader_unimplemented", module_loader_reason)
    } else if !candidate.allocator_authority.present {
        (
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_MISSING_STATUS,
            authority_reason,
        )
    } else if !candidate.allocation_intent.present
        || !candidate.allocation_intent.source_chain_complete
    {
        (
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
            allocation_intent_reason,
        )
    } else if !authority_inputs_complete {
        ("missing", first_authority_input_reason)
    } else if !candidate.authority_decision.present
        || !candidate.authority_decision.source_chain_complete
        || !candidate.authority_decision.input_chain_complete
    {
        (
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
            authority_decision_reason,
        )
    } else if !candidate.registry_write_commit_gate.present
        || !candidate.registry_write_commit_gate.source_chain_complete
    {
        (
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason,
        )
    } else {
        (
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
        )
    };

    ModuleServiceSlotAllocatorEvaluation {
        status,
        reason,
        retained_reservation_status,
        retained_reservation_reason,
        allocator_runtime_status,
        allocator_runtime_reason,
        registry_binding_status,
        registry_binding_reason,
        health_state_status,
        health_state_reason,
        unload_cleanup_status,
        unload_cleanup_reason,
        durable_audit_status,
        durable_audit_reason,
        rollback_status,
        rollback_reason,
        module_loader_status,
        module_loader_reason,
        authority_status,
        authority_reason,
        allocation_intent_status,
        allocation_intent_reason,
        authority_input_statuses,
        authority_input_reasons,
        authority_decision_status,
        authority_decision_reason,
        registry_write_commit_gate_status,
        registry_write_commit_gate_reason,
        allocates_service_slot: false,
        creates_service_inventory_records: false,
        can_allocate: false,
        can_load: false,
        load_attempted: false,
    }
}

fn module_service_slot_authority_input_status(
    input: ModuleServiceSlotAuthorityInput,
) -> &'static str {
    if input.present && input.source_chain_complete {
        input.spec.available_status
    } else {
        input.spec.missing_status
    }
}

fn module_service_slot_authority_input_reason(
    input: ModuleServiceSlotAuthorityInput,
) -> &'static str {
    if input.present && input.source_chain_complete {
        input.spec.available_reason
    } else {
        input.spec.source_chain_incomplete_reason
    }
}

fn evaluate_module_service_slot_allocator_fact(
    fact: ModuleServiceSlotAllocatorFact,
    scope_reason: &'static str,
    schema_reason: &'static str,
    missing_reason: &'static str,
    provenance_reason: &'static str,
    retained_reservation_reason: &'static str,
    allocator_runtime_reason: Option<&'static str>,
    available_reason: &'static str,
) -> (&'static str, &'static str) {
    if !method_eq(fact.scope, "current_boot") {
        return ("rejected", scope_reason);
    }
    if !fact.schema_ok {
        return ("rejected", schema_reason);
    }
    if !fact.present {
        return ("missing", missing_reason);
    }
    if !fact.provenance_ok {
        return ("rejected", provenance_reason);
    }
    if !fact.binds_retained_reservation {
        return ("rejected", retained_reservation_reason);
    }
    if let Some(reason) = allocator_runtime_reason {
        if !fact.binds_allocator_runtime {
            return ("rejected", reason);
        }
    }
    ("available", available_reason)
}

fn module_service_slot_allocator_observed_missing_fact(
    source: ModuleServiceSlotAllocatorFactSource,
    sequence: u64,
) -> ModuleServiceSlotAllocatorFact {
    ModuleServiceSlotAllocatorFact {
        source_evidence_event_id: Some(event_log::EventId { sequence }),
        source_evidence_state: "observed_current_boot_missing",
        source_evidence_status: "missing",
        source_evidence_reason: source.missing_reason,
        ..module_service_slot_allocator_missing_fact(source)
    }
}

fn module_service_slot_allocator_observed_available_fact(
    source: ModuleServiceSlotAllocatorFactSource,
    sequence: u64,
    available_reason: &'static str,
) -> ModuleServiceSlotAllocatorFact {
    ModuleServiceSlotAllocatorFact {
        source_evidence_event_id: Some(event_log::EventId { sequence }),
        source_evidence_state: "observed_current_boot_available",
        source_evidence_status: "available",
        source_evidence_reason: available_reason,
        ..module_service_slot_allocator_available_fact(source)
    }
}

fn module_service_slot_allocator_observed_missing_prerequisite(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
    sequence: u64,
) -> ModuleServiceSlotAllocatorPrerequisite {
    ModuleServiceSlotAllocatorPrerequisite {
        source_evidence_event_id: Some(event_log::EventId { sequence }),
        source_evidence_state: "observed_current_boot_missing",
        source_evidence_status: source.missing_status,
        source_evidence_reason: source.missing_reason,
        ..module_service_slot_allocator_missing_prerequisite(source)
    }
}

fn module_service_slot_allocator_observed_available_prerequisite(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
    sequence: u64,
    available_reason: &'static str,
) -> ModuleServiceSlotAllocatorPrerequisite {
    ModuleServiceSlotAllocatorPrerequisite {
        available: true,
        source_evidence_event_id: Some(event_log::EventId { sequence }),
        source_evidence_state: "observed_current_boot_available",
        source_evidence_status: "available",
        source_evidence_reason: available_reason,
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
        source_evidence_schema: source.source_evidence_schema,
    }
}

fn module_service_slot_allocator_selftest_cases(
) -> [ModuleServiceSlotAllocatorSelfTestCase; MODULE_SERVICE_SLOT_ALLOCATOR_SELFTEST_CASES] {
    let missing = module_service_slot_allocator_empty_snapshot(false);
    let allocator_available =
        module_service_slot_allocator_available_fact(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0]);
    let registry_available =
        module_service_slot_allocator_available_fact(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1]);
    let health_available =
        module_service_slot_allocator_available_fact(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2]);
    let unload_available =
        module_service_slot_allocator_available_fact(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3]);
    let durable_available = module_service_slot_allocator_available_prerequisite(
        MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0],
    );
    let rollback_available = module_service_slot_allocator_available_prerequisite(
        MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1],
    );
    let module_loader_available = module_service_slot_allocator_available_prerequisite(
        MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2],
    );
    let allocator_authority_available = module_service_slot_allocator_available_authority();
    let allocation_intent_available = module_service_slot_allocation_intent_available();
    let authority_inputs_available = module_service_slot_authority_inputs_available();
    let authority_decision_available = module_service_slot_allocator_authority_decision_available();
    let registry_write_commit_gate_available =
        module_service_slot_registry_write_commit_gate_available();
    let ready = ModuleServiceSlotAllocatorCandidate {
        retained_reservation_present: true,
        allocator_runtime: allocator_available,
        registry_binding: registry_available,
        health_state: health_available,
        unload_cleanup: unload_available,
        durable_audit_write: durable_available,
        rollback_plan_install: rollback_available,
        module_loader: module_loader_available,
        allocator_authority: allocator_authority_available,
        allocation_intent: allocation_intent_available,
        authority_inputs: authority_inputs_available,
        authority_decision: authority_decision_available,
        registry_write_commit_gate: registry_write_commit_gate_available,
    };
    [
        module_service_slot_allocator_selftest_case(
            "missing_retained_service_slot_reservation",
            "missing",
            "retained_service_slot_reservation_missing",
            missing,
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_allocator_previous_boot",
            "rejected",
            "service_slot_allocator_scope_must_be_current_boot",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: ModuleServiceSlotAllocatorFact {
                    scope: "previous_boot",
                    ..allocator_available
                },
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_allocator_wrong_schema",
            "rejected",
            "service_slot_allocator_schema_mismatch",
            ModuleServiceSlotAllocatorCandidate {
                allocator_runtime: ModuleServiceSlotAllocatorFact {
                    schema_ok: false,
                    ..allocator_available
                },
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_allocator_provenance_missing",
            "rejected",
            "service_slot_allocator_provenance_missing",
            ModuleServiceSlotAllocatorCandidate {
                allocator_runtime: ModuleServiceSlotAllocatorFact {
                    provenance_ok: false,
                    ..allocator_available
                },
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_allocator_retained_reservation_binding_missing",
            "rejected",
            "service_slot_allocator_retained_reservation_binding_missing",
            ModuleServiceSlotAllocatorCandidate {
                allocator_runtime: ModuleServiceSlotAllocatorFact {
                    binds_retained_reservation: false,
                    ..allocator_available
                },
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_allocator_runtime_missing",
            "missing",
            "service_slot_allocator_runtime_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_allocator_runtime_observed_source_evidence_missing",
            "missing",
            "service_slot_allocator_runtime_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: module_service_slot_allocator_observed_missing_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0],
                    42,
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_allocator_runtime_observed_source_evidence_available_registry_missing",
            "missing",
            "service_slot_registry_binding_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: module_service_slot_allocator_observed_available_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0],
                    49,
                    "service_slot_allocator_runtime_available",
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_registry_binding_missing",
            "missing",
            "service_slot_registry_binding_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_registry_binding_observed_source_evidence_missing",
            "missing",
            "service_slot_registry_binding_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                registry_binding: module_service_slot_allocator_observed_missing_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1],
                    43,
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_registry_binding_observed_source_evidence_available_health_missing",
            "missing",
            "service_health_state_model_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                registry_binding: module_service_slot_allocator_observed_available_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1],
                    50,
                    "service_slot_registry_binding_available",
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_registry_allocator_runtime_binding_missing",
            "rejected",
            "service_slot_registry_allocator_runtime_binding_missing",
            ModuleServiceSlotAllocatorCandidate {
                registry_binding: ModuleServiceSlotAllocatorFact {
                    binds_allocator_runtime: false,
                    ..registry_available
                },
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_health_state_model_missing",
            "missing",
            "service_health_state_model_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                registry_binding: registry_available,
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_health_state_model_observed_source_evidence_missing",
            "missing",
            "service_health_state_model_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                registry_binding: registry_available,
                health_state: module_service_slot_allocator_observed_missing_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2],
                    44,
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_health_state_model_observed_source_evidence_available_unload_missing",
            "missing",
            "service_unload_cleanup_plan_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                registry_binding: registry_available,
                health_state: module_service_slot_allocator_observed_available_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2],
                    51,
                    "service_health_state_model_available",
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_unload_cleanup_plan_missing",
            "missing",
            "service_unload_cleanup_plan_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                registry_binding: registry_available,
                health_state: health_available,
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_unload_cleanup_plan_observed_source_evidence_missing",
            "missing",
            "service_unload_cleanup_plan_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                registry_binding: registry_available,
                health_state: health_available,
                unload_cleanup: module_service_slot_allocator_observed_missing_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3],
                    45,
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_unload_cleanup_plan_observed_source_evidence_available_durable_missing",
            "denied_missing_durable_audit_write",
            "durable_audit_write_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                registry_binding: registry_available,
                health_state: health_available,
                unload_cleanup: module_service_slot_allocator_observed_available_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3],
                    52,
                    "service_unload_cleanup_plan_available",
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "durable_audit_write_missing",
            "denied_missing_durable_audit_write",
            "durable_audit_write_missing",
            ModuleServiceSlotAllocatorCandidate {
                durable_audit_write: module_service_slot_allocator_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0],
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "durable_audit_write_observed_source_evidence_missing",
            "denied_missing_durable_audit_write",
            "durable_audit_write_missing",
            ModuleServiceSlotAllocatorCandidate {
                durable_audit_write: module_service_slot_allocator_observed_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0],
                    46,
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "durable_audit_write_observed_source_evidence_available_rollback_missing",
            "denied_missing_rollback_install",
            "rollback_install_missing",
            ModuleServiceSlotAllocatorCandidate {
                durable_audit_write: module_service_slot_allocator_observed_available_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0],
                    53,
                    "durable_audit_write_available",
                ),
                rollback_plan_install: module_service_slot_allocator_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1],
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "rollback_install_missing",
            "denied_missing_rollback_install",
            "rollback_install_missing",
            ModuleServiceSlotAllocatorCandidate {
                rollback_plan_install: module_service_slot_allocator_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1],
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "rollback_install_observed_source_evidence_missing",
            "denied_missing_rollback_install",
            "rollback_install_missing",
            ModuleServiceSlotAllocatorCandidate {
                rollback_plan_install: module_service_slot_allocator_observed_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1],
                    47,
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "rollback_install_observed_source_evidence_available_module_loader_unimplemented",
            "denied_loader_unimplemented",
            "module_loader_unimplemented",
            ModuleServiceSlotAllocatorCandidate {
                durable_audit_write: durable_available,
                rollback_plan_install:
                    module_service_slot_allocator_observed_available_prerequisite(
                        MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1],
                        54,
                        "rollback_plan_install_available",
                    ),
                module_loader: module_service_slot_allocator_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2],
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "module_loader_missing",
            "denied_loader_unimplemented",
            "module_loader_unimplemented",
            ModuleServiceSlotAllocatorCandidate {
                module_loader: module_service_slot_allocator_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2],
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "module_loader_observed_source_evidence_missing",
            "denied_loader_unimplemented",
            "module_loader_unimplemented",
            ModuleServiceSlotAllocatorCandidate {
                module_loader: module_service_slot_allocator_observed_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2],
                    48,
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "module_loader_observed_source_evidence_available_allocator_authority_boundary",
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
            ModuleServiceSlotAllocatorCandidate {
                module_loader: module_service_slot_allocator_observed_available_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2],
                    55,
                    "module_loader_boundary_available_non_authorizing",
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "registry_write_commit_gate_missing",
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_CHAIN_INCOMPLETE_REASON,
            ModuleServiceSlotAllocatorCandidate {
                registry_write_commit_gate: module_service_slot_registry_write_commit_gate_missing(
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "all_inputs_ready_still_non_authorizing",
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
            ready,
        ),
    ]
}

fn module_service_slot_allocator_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleServiceSlotAllocatorCandidate,
) -> ModuleServiceSlotAllocatorSelfTestCase {
    let actual = evaluate_module_service_slot_allocator_candidate(candidate);
    ModuleServiceSlotAllocatorSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_allocator_runtime_source_evidence_present: candidate
            .allocator_runtime
            .source_evidence_event_id
            .is_some(),
        actual_allocator_runtime_source_evidence_state: candidate
            .allocator_runtime
            .source_evidence_state,
        actual_allocator_runtime_source_evidence_status: candidate
            .allocator_runtime
            .source_evidence_status,
        actual_allocator_runtime_source_evidence_reason: candidate
            .allocator_runtime
            .source_evidence_reason,
        actual_registry_binding_source_evidence_present: candidate
            .registry_binding
            .source_evidence_event_id
            .is_some(),
        actual_registry_binding_source_evidence_state: candidate
            .registry_binding
            .source_evidence_state,
        actual_registry_binding_source_evidence_status: candidate
            .registry_binding
            .source_evidence_status,
        actual_registry_binding_source_evidence_reason: candidate
            .registry_binding
            .source_evidence_reason,
        actual_health_state_source_evidence_present: candidate
            .health_state
            .source_evidence_event_id
            .is_some(),
        actual_health_state_source_evidence_state: candidate.health_state.source_evidence_state,
        actual_health_state_source_evidence_status: candidate.health_state.source_evidence_status,
        actual_health_state_source_evidence_reason: candidate.health_state.source_evidence_reason,
        actual_unload_cleanup_source_evidence_present: candidate
            .unload_cleanup
            .source_evidence_event_id
            .is_some(),
        actual_unload_cleanup_source_evidence_state: candidate.unload_cleanup.source_evidence_state,
        actual_unload_cleanup_source_evidence_status: candidate
            .unload_cleanup
            .source_evidence_status,
        actual_unload_cleanup_source_evidence_reason: candidate
            .unload_cleanup
            .source_evidence_reason,
        actual_durable_audit_source_evidence_present: candidate
            .durable_audit_write
            .source_evidence_event_id
            .is_some(),
        actual_durable_audit_source_evidence_state: candidate
            .durable_audit_write
            .source_evidence_state,
        actual_durable_audit_source_evidence_status: candidate
            .durable_audit_write
            .source_evidence_status,
        actual_durable_audit_source_evidence_reason: candidate
            .durable_audit_write
            .source_evidence_reason,
        actual_rollback_install_source_evidence_present: candidate
            .rollback_plan_install
            .source_evidence_event_id
            .is_some(),
        actual_rollback_install_source_evidence_state: candidate
            .rollback_plan_install
            .source_evidence_state,
        actual_rollback_install_source_evidence_status: candidate
            .rollback_plan_install
            .source_evidence_status,
        actual_rollback_install_source_evidence_reason: candidate
            .rollback_plan_install
            .source_evidence_reason,
        actual_module_loader_source_evidence_present: candidate
            .module_loader
            .source_evidence_event_id
            .is_some(),
        actual_module_loader_source_evidence_state: candidate.module_loader.source_evidence_state,
        actual_module_loader_source_evidence_status: candidate.module_loader.source_evidence_status,
        actual_module_loader_source_evidence_reason: candidate.module_loader.source_evidence_reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && module_service_slot_allocator_source_fact_map_complete()
            && module_service_slot_allocator_prerequisite_source_map_complete()
            && !actual.allocates_service_slot
            && !actual.creates_service_inventory_records
            && !actual.can_allocate
            && !actual.can_load
            && !actual.load_attempted,
    }
}
