use alloc::vec;

use crate::{
    agent_protocol_module_audit::module_audit_rollback_valid_input,
    agent_protocol_module_reference::{
        common_evidence_status, diagnostic_facts, emit_evidence_v1_response, evidence_record,
        selftest_case, selftest_facts,
    },
    agent_protocol_module_types::*,
    agent_protocol_support::{
        current_boot_event_id_str, method_eq, method_head_eq, parse_current_boot_event_id,
        parse_sha256_ref, record_bool as b, record_event_or_null, record_false as no,
        record_field as f, record_gate, record_present_absent, record_sha_fields,
        record_sha_or_null, record_str as s, record_str_or_null, run_selftest_cases_with, CaseSpec,
    },
    event_log, granted_candidate_service,
    module_evidence::{
        self, ram_only_service_slot_id_valid, ModuleServiceSlotReservationHashInput,
    },
};
use raios_core::evidence_response as ev;
use raios_core::evidence_response::Blocked;
use raios_core::record::Value as V;

#[derive(Clone, Copy)]
enum ServiceSlotSelftestMutation {
    Absent,
    Accepted,
    Stale,
    MismatchedReservationHash,
    InvalidRamOnlyServiceSlot,
}

const fn service_slot_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    mutation: ServiceSlotSelftestMutation,
) -> CaseSpec<ServiceSlotSelftestMutation> {
    CaseSpec {
        name,
        expected_status,
        expected_reason,
        mutation,
        require_live_retained: false,
    }
}

const SERVICE_SLOT_CASES: [CaseSpec<ServiceSlotSelftestMutation>;
    MODULE_SERVICE_SLOT_SELFTEST_CASES] = [
    service_slot_case(
        "absent_reference",
        "missing",
        "service_slot_reservation_reference_absent",
        ServiceSlotSelftestMutation::Absent,
    ),
    service_slot_case(
        "accepted_current_boot_reservation_still_denied",
        "valid_hash_reference_load_still_denied",
        "service_slot_reservation_valid_but_allocator_and_loader_missing",
        ServiceSlotSelftestMutation::Accepted,
    ),
    service_slot_case(
        "stale_previous_boot_reservation",
        "stale_or_non_current_boot_reference",
        "service_slot_reservation_scope_must_be_current_boot",
        ServiceSlotSelftestMutation::Stale,
    ),
    service_slot_case(
        "mismatched_reservation_hash",
        "mismatched_reservation_hash",
        "service_slot_reservation_hash_mismatch",
        ServiceSlotSelftestMutation::MismatchedReservationHash,
    ),
    service_slot_case(
        "invalid_ram_only_service_slot",
        "rejected",
        "ram_only_service_slot_id_invalid",
        ServiceSlotSelftestMutation::InvalidRamOnlyServiceSlot,
    ),
];

fn module_service_slot_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "module.service_slot_diagnostic") {
        "module.service_slot_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

pub(crate) fn emit_module_service_slot_diagnostic(method: &str) {
    let arg = module_service_slot_diagnostic_arg(method);
    let check = parse_module_service_slot_reservation(arg, true);
    let recorded_event_id = if check.valid {
        module_service_slot_binding_from_check(&check)
            .map(event_log::record_module_service_slot_reservation)
    } else {
        None
    };
    let retained = event_log::latest_module_service_slot_reservation();
    let retained_slot_id = retained.map(|(_, reference)| reference.ram_only_service_slot_id);

    let live_snapshot = granted_candidate_service::loaded_snapshot();
    let facts = diagnostic_facts("module.service_slot_diagnostic <reservation_hash> <retained_reference_event_id> <retained_audit_rollback_reference_event_id> <computed_grant_hash> <audit_record_hash> <rollback_plan_hash> <pre_load_service_inventory_hash> <ram_only_service_slot_id> [current_boot]",
        V::InlineObject(vec![f("requested_capability", s("cap.module.load_ephemeral")), f("load_mode", s("ram_only")), f("subject", s("agent.session.serial")), f("resource", s("live_service_graph"))]),
        "hash_reference_only_no_slot_allocation", V::InlineArray(vec![s("service_slot_allocator"), s("module_loader")]), V::Null,
        V::InlineObject(vec![f("live_granted_service_slot_present", b(live_snapshot.is_some())), f("service_slot_allocator", s("unavailable")), f("loader", s("unavailable"))]));
    let mut evidence = vec![evidence_record(
        "service_slot_reservation",
        "reference",
        common_evidence_status(check.valid, check.has_reference),
        check.reason,
        None,
        V::InlineObject(vec![
            f("state", record_present_absent(check.has_reference)),
            f("status_detail", s(check.status)),
            f("arity_valid", b(check.arity_valid)),
            f("scope", s(check.scope)),
            f(
                "reservation_hash",
                record_sha_or_null(check.reservation_hash),
            ),
            f(
                "expected_reservation_hash",
                record_sha_or_null(check.expected_reservation_hash),
            ),
            f(
                "retained_computed_grant_reference_event_id",
                record_str_or_null(check.retained_reference_event_id),
            ),
            f(
                "retained_audit_rollback_reference_event_id",
                record_str_or_null(check.retained_audit_rollback_reference_event_id),
            ),
            f(
                "computed_capability_grant_hash",
                record_sha_or_null(check.computed_grant_hash),
            ),
            f(
                "audit_record_hash",
                record_sha_or_null(check.audit_record_hash),
            ),
            f(
                "rollback_plan_hash",
                record_sha_or_null(check.rollback_plan_hash),
            ),
            f(
                "pre_load_service_inventory_hash",
                record_sha_or_null(check.pre_load_service_inventory_hash),
            ),
            f(
                "ram_only_service_slot_id",
                record_str_or_null(check.ram_only_service_slot_id),
            ),
        ]),
    )];
    if let Some((event_id, reference)) = retained {
        evidence.push(evidence_record(
            "service_slot_reservation_retained",
            "retained_reference",
            "verified",
            "retained_hash_reference_load_still_denied",
            Some(event_id),
            V::InlineObject(vec![
                f("state", s("present")),
                f("retention", s("current_boot_ram_event_log")),
                f(
                    "matches_current_reference",
                    b(module_service_slot_reference_matches(&check, reference)),
                ),
                f(
                    "record_schema",
                    s("raios.module_service_slot_reservation.v0"),
                ),
                f(
                    "status_detail",
                    s("retained_hash_reference_load_still_denied"),
                ),
                f("reservation_hash", V::Sha256(reference.reservation_hash)),
                f(
                    "computed_capability_grant_hash",
                    V::Sha256(reference.computed_grant_hash),
                ),
                f("audit_record_hash", V::Sha256(reference.audit_record_hash)),
                f(
                    "rollback_plan_hash",
                    V::Sha256(reference.rollback_plan_hash),
                ),
                f(
                    "pre_load_service_inventory_hash",
                    V::Sha256(reference.pre_load_service_inventory_hash),
                ),
                f(
                    "ram_only_service_slot_id",
                    s(retained_slot_id.as_ref().unwrap().as_str()),
                ),
            ]),
        ));
    } else {
        evidence.push(evidence_record(
            "service_slot_reservation_retained",
            "retained_reference",
            "missing",
            "no_valid_service_slot_reservation_retained",
            None,
            V::InlineObject(vec![
                f("state", s("missing")),
                f("retention", s("current_boot_ram_event_log")),
                f("matches_current_reference", no()),
                f(
                    "record_schema",
                    s("raios.module_service_slot_reservation.v0"),
                ),
                f("status_detail", s("missing")),
            ]),
        ));
    }
    if let Some(snapshot) = live_snapshot {
        evidence.push(evidence_record(
            "live_granted_service_slot",
            "runtime_snapshot",
            "present",
            "live_service_slot_snapshot",
            None,
            V::InlineObject(vec![
                f("state", s("allocated")),
                f(
                    "service_id",
                    s(granted_candidate_service::GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.service_id),
                ),
                f(
                    "ram_only_service_slot_id",
                    s(granted_candidate_service::ram_only_service_slot_id()),
                ),
                f("service_slot_allocated", b(snapshot.loaded)),
                f("running", b(snapshot.running)),
                f(
                    "health",
                    s(granted_candidate_service::health_state(snapshot)),
                ),
                f(
                    "service_slot_activation_id",
                    s(granted_candidate_service::service_slot_activation_id()),
                ),
                f(
                    "service_slot_activation_hash",
                    V::Sha256(granted_candidate_service::service_slot_activation_hash()),
                ),
                f(
                    "service_slot_activation_status",
                    s(granted_candidate_service::service_slot_activation_status(
                        snapshot,
                    )),
                ),
                f(
                    "service_slot_activation_active",
                    b(granted_candidate_service::service_slot_activation_active(
                        snapshot,
                    )),
                ),
                f("trust_tier", s(snapshot.trust_tier)),
                f("load_mechanism", s("wasmi_interpreter_ram_only")),
                f("maps_executable_pages", no()),
                f("durable", no()),
                f("owner_sealed", no()),
            ]),
        ));
    }
    let primary = (!check.valid).then_some(Blocked {
        evidence_id: "service_slot_reservation",
        status: common_evidence_status(false, check.has_reference),
        reason: check.reason,
    });
    emit_evidence_v1_response(
        "module.service_slot_diagnostic",
        "module.service_slot_reservation",
        recorded_event_id,
        facts,
        evidence,
        ev::module_reference_denial(ev::ModuleReferenceFamily::ServiceSlot, primary),
    );
    return;
}

pub(crate) fn emit_module_service_slot_diagnostic_selftest() {
    let cases = module_service_slot_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }
    let case_records = cases
        .iter()
        .map(module_service_slot_selftest_case_record)
        .collect();

    emit_evidence_v1_response(
        "module.service_slot_diagnostic_selftest",
        "module.service_slot_reservation.selftest",
        None,
        selftest_facts(V::Array(case_records), cases.len(), passed),
        vec![],
        ev::observed("selftest_completed"),
    );
}

#[rustfmt::skip]
fn module_service_slot_selftest_case_record(case: &ModuleServiceSlotSelfTestCase) -> V<'static> {
    selftest_case(case.name, case.expected_status, case.expected_reason, case.actual_status, case.actual_reason, case.passed)
}

fn parse_module_service_slot_reservation(
    arg: &str,
    require_live_retained: bool,
) -> ModuleServiceSlotReservationCheck<'_> {
    let arg = arg.trim();
    if arg.is_empty() {
        return evaluate_module_service_slot_reservation(
            ModuleServiceSlotReservationInput {
                has_reference: false,
                arity_valid: true,
                scope: "current_boot",
                reservation_hash: None,
                retained_reference_event_id: None,
                retained_audit_rollback_reference_event_id: None,
                computed_grant_hash: None,
                audit_record_hash: None,
                rollback_plan_hash: None,
                pre_load_service_inventory_hash: None,
                ram_only_service_slot_id: None,
            },
            require_live_retained,
        );
    }
    let mut parts = arg.split_whitespace();
    let reservation_token = parts.next();
    let retained_reference_event_id = parts.next();
    let retained_audit_rollback_reference_event_id = parts.next();
    let grant_token = parts.next();
    let audit_token = parts.next();
    let rollback_token = parts.next();
    let inventory_token = parts.next();
    let service_slot_token = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let arity_valid = reservation_token.is_some()
        && retained_reference_event_id.is_some()
        && retained_audit_rollback_reference_event_id.is_some()
        && grant_token.is_some()
        && audit_token.is_some()
        && rollback_token.is_some()
        && inventory_token.is_some()
        && service_slot_token.is_some()
        && extra.is_none();

    evaluate_module_service_slot_reservation(
        ModuleServiceSlotReservationInput {
            has_reference: true,
            arity_valid,
            scope,
            reservation_hash: reservation_token.and_then(parse_sha256_ref),
            retained_reference_event_id,
            retained_audit_rollback_reference_event_id,
            computed_grant_hash: grant_token.and_then(parse_sha256_ref),
            audit_record_hash: audit_token.and_then(parse_sha256_ref),
            rollback_plan_hash: rollback_token.and_then(parse_sha256_ref),
            pre_load_service_inventory_hash: inventory_token.and_then(parse_sha256_ref),
            ram_only_service_slot_id: service_slot_token,
        },
        require_live_retained,
    )
}

fn evaluate_module_service_slot_reservation<'a>(
    input: ModuleServiceSlotReservationInput<'a>,
    require_live_retained: bool,
) -> ModuleServiceSlotReservationCheck<'a> {
    if !input.has_reference {
        return module_service_slot_reservation_check(
            input,
            None,
            "missing",
            "service_slot_reservation_reference_absent",
            false,
        );
    }
    if !input.arity_valid {
        return module_service_slot_reservation_check(
            input,
            None,
            "invalid_reference_arity",
            "service_slot_reservation_requires_hashes_events_slot_and_optional_scope",
            false,
        );
    }

    let (
        Some(reservation_hash),
        Some(retained_reference_event_id),
        Some(retained_audit_rollback_reference_event_id),
        Some(computed_grant_hash),
        Some(audit_record_hash),
        Some(rollback_plan_hash),
        Some(pre_load_service_inventory_hash),
        Some(ram_only_service_slot_id),
    ) = (
        input.reservation_hash,
        input.retained_reference_event_id,
        input.retained_audit_rollback_reference_event_id,
        input.computed_grant_hash,
        input.audit_record_hash,
        input.rollback_plan_hash,
        input.pre_load_service_inventory_hash,
        input.ram_only_service_slot_id,
    )
    else {
        return module_service_slot_reservation_check(
            input,
            None,
            "invalid_hash_reference",
            "all_service_slot_reservation_references_must_be_sha256_or_current_boot_ids",
            false,
        );
    };

    let expected_reservation_hash =
        computed_module_service_slot_reservation_hash(ModuleServiceSlotReservationHashInput {
            retained_reference_event_id,
            retained_audit_rollback_reference_event_id,
            computed_grant_hash,
            audit_record_hash,
            rollback_plan_hash,
            pre_load_service_inventory_hash,
            ram_only_service_slot_id,
        });

    if !method_eq(input.scope, "current_boot") {
        return module_service_slot_reservation_check(
            input,
            Some(expected_reservation_hash),
            "stale_or_non_current_boot_reference",
            "service_slot_reservation_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_reference_event_id) {
        return module_service_slot_reservation_check(
            input,
            Some(expected_reservation_hash),
            "rejected",
            "retained_reference_event_id_not_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(retained_audit_rollback_reference_event_id) {
        return module_service_slot_reservation_check(
            input,
            Some(expected_reservation_hash),
            "rejected",
            "retained_audit_rollback_reference_event_id_not_current_boot",
            false,
        );
    }
    if !ram_only_service_slot_id_valid(ram_only_service_slot_id) {
        return module_service_slot_reservation_check(
            input,
            Some(expected_reservation_hash),
            "rejected",
            "ram_only_service_slot_id_invalid",
            false,
        );
    }
    if reservation_hash != expected_reservation_hash {
        return module_service_slot_reservation_check(
            input,
            Some(expected_reservation_hash),
            "mismatched_reservation_hash",
            "service_slot_reservation_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = module_service_slot_live_reference_mismatch(&input) {
            return module_service_slot_reservation_check(
                input,
                Some(expected_reservation_hash),
                "rejected",
                reason,
                false,
            );
        }
    }

    module_service_slot_reservation_check(
        input,
        Some(expected_reservation_hash),
        "valid_hash_reference_load_still_denied",
        "service_slot_reservation_valid_but_allocator_and_loader_missing",
        true,
    )
}

fn module_service_slot_reservation_check<'a>(
    input: ModuleServiceSlotReservationInput<'a>,
    expected_reservation_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> ModuleServiceSlotReservationCheck<'a> {
    ModuleServiceSlotReservationCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        reservation_hash: input.reservation_hash,
        retained_reference_event_id: input.retained_reference_event_id,
        retained_audit_rollback_reference_event_id: input
            .retained_audit_rollback_reference_event_id,
        computed_grant_hash: input.computed_grant_hash,
        audit_record_hash: input.audit_record_hash,
        rollback_plan_hash: input.rollback_plan_hash,
        pre_load_service_inventory_hash: input.pre_load_service_inventory_hash,
        ram_only_service_slot_id: input.ram_only_service_slot_id,
        expected_reservation_hash,
        status,
        reason,
        valid,
    }
}

fn module_service_slot_live_reference_mismatch(
    input: &ModuleServiceSlotReservationInput<'_>,
) -> Option<&'static str> {
    let retained_reference_event_id =
        parse_current_boot_event_id(input.retained_reference_event_id?)?;
    let retained_audit_rollback_reference_event_id =
        parse_current_boot_event_id(input.retained_audit_rollback_reference_event_id?)?;
    let (latest_retained_event_id, retained_reference) =
        event_log::latest_module_computed_grant_reference()?;
    if latest_retained_event_id != retained_reference_event_id {
        return Some("service_slot_retained_computed_grant_reference_mismatch");
    }
    if Some(retained_reference.computed_grant_hash) != input.computed_grant_hash {
        return Some("service_slot_computed_grant_hash_mismatch");
    }

    let (latest_audit_event_id, audit_reference) =
        event_log::latest_module_audit_rollback_reference()?;
    if latest_audit_event_id != retained_audit_rollback_reference_event_id {
        return Some("service_slot_retained_audit_rollback_reference_mismatch");
    }
    if Some(audit_reference.computed_grant_hash) != input.computed_grant_hash {
        return Some("service_slot_computed_grant_hash_mismatch");
    }
    if Some(audit_reference.audit_record_hash) != input.audit_record_hash {
        return Some("service_slot_audit_record_hash_mismatch");
    }
    if Some(audit_reference.rollback_plan_hash) != input.rollback_plan_hash {
        return Some("service_slot_rollback_plan_hash_mismatch");
    }
    if Some(audit_reference.pre_load_service_inventory_hash)
        != input.pre_load_service_inventory_hash
    {
        return Some("service_slot_pre_load_inventory_hash_mismatch");
    }
    if Some(audit_reference.ram_only_service_slot_id.as_str()) != input.ram_only_service_slot_id {
        return Some("service_slot_id_mismatch");
    }

    None
}

fn module_service_slot_binding_from_check(
    check: &ModuleServiceSlotReservationCheck<'_>,
) -> Option<event_log::ModuleServiceSlotReservation> {
    Some(event_log::ModuleServiceSlotReservation {
        reservation_hash: check.reservation_hash?,
        retained_reference_event_id: parse_current_boot_event_id(
            check.retained_reference_event_id?,
        )?,
        retained_audit_rollback_reference_event_id: parse_current_boot_event_id(
            check.retained_audit_rollback_reference_event_id?,
        )?,
        computed_grant_hash: check.computed_grant_hash?,
        audit_record_hash: check.audit_record_hash?,
        rollback_plan_hash: check.rollback_plan_hash?,
        pre_load_service_inventory_hash: check.pre_load_service_inventory_hash?,
        ram_only_service_slot_id: event_log::ModuleServiceSlotId::new(
            check.ram_only_service_slot_id?,
        )?,
    })
}

fn module_service_slot_reference_matches(
    check: &ModuleServiceSlotReservationCheck<'_>,
    reference: event_log::ModuleServiceSlotReservation,
) -> bool {
    check.reservation_hash == Some(reference.reservation_hash)
        && check
            .retained_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_reference_event_id)
        && check
            .retained_audit_rollback_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_audit_rollback_reference_event_id)
        && check.computed_grant_hash == Some(reference.computed_grant_hash)
        && check.audit_record_hash == Some(reference.audit_record_hash)
        && check.rollback_plan_hash == Some(reference.rollback_plan_hash)
        && check.pre_load_service_inventory_hash == Some(reference.pre_load_service_inventory_hash)
        && check.ram_only_service_slot_id == Some(reference.ram_only_service_slot_id.as_str())
}

fn module_service_slot_selftest_cases(
) -> [ModuleServiceSlotSelfTestCase; MODULE_SERVICE_SLOT_SELFTEST_CASES] {
    run_selftest_cases_with(
        module_service_slot_valid_input(),
        &SERVICE_SLOT_CASES,
        apply_service_slot_selftest_case,
        evaluate_service_slot_selftest_case,
        module_service_slot_selftest_case_from_spec,
    )
}

fn apply_service_slot_selftest_case(
    candidate: &mut ModuleServiceSlotReservationInput<'static>,
    mutation: ServiceSlotSelftestMutation,
) {
    let valid = module_service_slot_valid_input();
    *candidate = match mutation {
        ServiceSlotSelftestMutation::Absent => ModuleServiceSlotReservationInput {
            has_reference: false,
            arity_valid: true,
            scope: "current_boot",
            reservation_hash: None,
            retained_reference_event_id: None,
            retained_audit_rollback_reference_event_id: None,
            computed_grant_hash: None,
            audit_record_hash: None,
            rollback_plan_hash: None,
            pre_load_service_inventory_hash: None,
            ram_only_service_slot_id: None,
        },
        ServiceSlotSelftestMutation::Accepted => valid,
        ServiceSlotSelftestMutation::Stale => ModuleServiceSlotReservationInput {
            scope: "previous_boot",
            ..valid
        },
        ServiceSlotSelftestMutation::MismatchedReservationHash => {
            ModuleServiceSlotReservationInput {
                reservation_hash: Some([0x99; 32]),
                ..valid
            }
        }
        ServiceSlotSelftestMutation::InvalidRamOnlyServiceSlot => {
            ModuleServiceSlotReservationInput {
                ram_only_service_slot_id: Some("svc.test.0001"),
                ..valid
            }
        }
    }
}

fn evaluate_service_slot_selftest_case(
    candidate: ModuleServiceSlotReservationInput<'_>,
    _require_live_retained: bool,
) -> ModuleServiceSlotReservationCheck<'_> {
    evaluate_module_service_slot_reservation(candidate, false)
}

fn module_service_slot_valid_input<'a>() -> ModuleServiceSlotReservationInput<'a> {
    let audit_rollback = module_audit_rollback_valid_input();
    let computed_grant_hash = audit_rollback.computed_grant_hash.unwrap_or([0; 32]);
    let audit_record_hash = audit_rollback.audit_record_hash.unwrap_or([0; 32]);
    let rollback_plan_hash = audit_rollback.rollback_plan_hash.unwrap_or([0; 32]);
    let pre_load_service_inventory_hash = audit_rollback
        .pre_load_service_inventory_hash
        .unwrap_or([0; 32]);
    let reservation_hash =
        computed_module_service_slot_reservation_hash(ModuleServiceSlotReservationHashInput {
            retained_reference_event_id: MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
            retained_audit_rollback_reference_event_id:
                MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID,
            computed_grant_hash,
            audit_record_hash,
            rollback_plan_hash,
            pre_load_service_inventory_hash,
            ram_only_service_slot_id: MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
        });
    ModuleServiceSlotReservationInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        reservation_hash: Some(reservation_hash),
        retained_reference_event_id: Some(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID),
        retained_audit_rollback_reference_event_id: Some(
            MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID,
        ),
        computed_grant_hash: Some(computed_grant_hash),
        audit_record_hash: Some(audit_record_hash),
        rollback_plan_hash: Some(rollback_plan_hash),
        pre_load_service_inventory_hash: Some(pre_load_service_inventory_hash),
        ram_only_service_slot_id: Some(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID),
    }
}

fn module_service_slot_selftest_case_from_spec(
    spec: &CaseSpec<ServiceSlotSelftestMutation>,
    actual: ModuleServiceSlotReservationCheck<'_>,
) -> ModuleServiceSlotSelfTestCase {
    ModuleServiceSlotSelfTestCase {
        name: spec.name,
        expected_status: spec.expected_status,
        expected_reason: spec.expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, spec.expected_status)
            && method_eq(actual.reason, spec.expected_reason)
            && actual.valid
                == method_eq(
                    spec.expected_status,
                    "valid_hash_reference_load_still_denied",
                ),
    }
}

fn computed_module_service_slot_reservation_hash(
    input: ModuleServiceSlotReservationHashInput<'_>,
) -> [u8; 32] {
    module_evidence::computed_module_service_slot_reservation_hash(input)
}
