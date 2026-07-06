use alloc::vec;

use crate::{
    agent_protocol_module_audit::module_audit_rollback_valid_input,
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, current_boot_event_id_str, emit_record_fields_trailing_comma,
        emit_record_property_line, emit_record_value_property_line, end_response, method_eq,
        method_head_eq, parse_current_boot_event_id, parse_sha256_ref, record_bool as b,
        record_event_or_null, record_false as no, record_field as f, record_gate,
        record_sha_fields, record_sha_or_null, record_str as s, record_str_or_null,
        run_selftest_cases_with, CaseSpec,
    },
    event_log, granted_candidate_service,
    module_evidence::{
        self, ram_only_service_slot_id_valid, ModuleServiceSlotReservationHashInput,
    },
};
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

#[rustfmt::skip]
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

    begin_response("module.service_slot_diagnostic");
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_service_slot_reservation_diagnostic.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", no()),
            f("mutates_global_event_log", b(check.valid)),
            f(
                "global_event_log_mutation",
                s(if check.valid {
                    "valid_hash_reference_retention_only"
                } else {
                    "none"
                }),
            ),
            f("accepts_artifact_bytes", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("loads_artifact", no()),
            f(
                "reference_format",
                s("module.service_slot_diagnostic <reservation_hash> <retained_reference_event_id> <retained_audit_rollback_reference_event_id> <computed_grant_hash> <audit_record_hash> <rollback_plan_hash> <pre_load_service_inventory_hash> <ram_only_service_slot_id> [current_boot]"),
            ),
        ],
        6,
    );
    emit_module_service_slot_reference_object(&check, true);
    emit_module_service_slot_retained_reference(&check, recorded_event_id, retained, true);
    emit_module_service_slot_policy_result(&check, true);
    if let Some(snapshot) = granted_candidate_service::loaded_snapshot() {
        emit_live_granted_service_slot(snapshot, true);
    }
    emit_record_value_property_line(
        "blocked_by",
        V::Array(vec![
            record_gate("service_slot_allocator", "unavailable",
                "ram_only_service_slot_allocator_unimplemented"),
            record_gate("module_loader", "unavailable", "module_loader_unimplemented"),
        ]),
        false,
    );
    end_response("module.service_slot_diagnostic");
}

fn emit_live_granted_service_slot(snapshot: granted_candidate_service::Snapshot, comma: bool) {
    let projection = granted_candidate_service::live_load_projection();
    emit_record_property_line(
        "live_granted_service_slot",
        vec![
            f("state", s("allocated")),
            f(
                "service_id",
                s(granted_candidate_service::GRANTED_CANDIDATE_SERVICE_DESCRIPTOR.service_id),
            ),
            f(
                "ram_only_service_slot_id",
                s(granted_candidate_service::ram_only_service_slot_id()),
            ),
            f(
                "service_slot_allocated",
                b(projection.service_slot_allocated),
            ),
            f("running", b(projection.running)),
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
                record_sha_or_null(Some(
                    granted_candidate_service::service_slot_activation_hash(),
                )),
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
            f("trust_tier", s(projection.trust_tier)),
            f("load_mechanism", s(projection.load_mechanism)),
            f("maps_executable_pages", no()),
            f("durable", no()),
            f("owner_sealed", no()),
            f("authorizes_native_guest_load", no()),
        ],
        comma,
    );
}

#[rustfmt::skip]
fn emit_module_service_slot_reference_object(
    check: &ModuleServiceSlotReservationCheck<'_>,
    comma: bool,
) {
    emit_record_property_line(
        "service_slot_reservation_reference",
        vec![
            f(
                "state",
                s(if check.has_reference { "present" } else { "absent" }),
            ),
            f("validation_status", s(check.status)),
            f("validation_reason", s(check.reason)),
            f("arity_valid", b(check.arity_valid)),
            f("scope", s(check.scope)),
            f("reservation_hash", record_sha_or_null(check.reservation_hash)),
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
            f("audit_record_hash", record_sha_or_null(check.audit_record_hash)),
            f("rollback_plan_hash", record_sha_or_null(check.rollback_plan_hash)),
            f(
                "pre_load_service_inventory_hash",
                record_sha_or_null(check.pre_load_service_inventory_hash),
            ),
            f(
                "ram_only_service_slot_id",
                record_str_or_null(check.ram_only_service_slot_id),
            ),
        ],
        comma,
    );
}

#[rustfmt::skip]
fn emit_module_service_slot_retained_reference(
    check: &ModuleServiceSlotReservationCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(event_log::EventId, event_log::ModuleServiceSlotReservation)>,
    comma: bool,
) {
    let fields = if let Some((event_id, ref reference)) = retained {
        vec![
            f("state", s("present")),
            f("retention", s("current_boot_ram_event_log")),
            f("event_id", record_event_or_null(Some(event_id))),
            f("recorded_event_id", record_event_or_null(recorded_event_id)),
            f(
                "matches_current_reference",
                b(module_service_slot_reference_matches(check, *reference)),
            ),
            f("schema", s("raios.module_service_slot_reservation.v0")),
            f("status", s("retained_hash_reference_load_still_denied")),
            f("classification", s("local_only")),
            f("allocates_service_slot", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
            f(
                "retained_computed_grant_reference_event_id",
                record_event_or_null(Some(reference.retained_reference_event_id)),
            ),
            f(
                "retained_audit_rollback_reference_event_id",
                record_event_or_null(Some(reference.retained_audit_rollback_reference_event_id)),
            ),
            f("ram_only_service_slot_id", s(reference.ram_only_service_slot_id.as_str())),
            f(
                "hashes",
                V::Object(record_sha_fields(&[
                    ("reservation_hash", reference.reservation_hash),
                    ("computed_capability_grant_hash", reference.computed_grant_hash),
                    ("audit_record_hash", reference.audit_record_hash),
                    ("rollback_plan_hash", reference.rollback_plan_hash),
                    ("pre_load_service_inventory_hash", reference.pre_load_service_inventory_hash),
                ])),
            ),
        ]
    } else {
        vec![
            f("state", s("missing")),
            f("retention", s("current_boot_ram_event_log")),
            f("event_id", record_event_or_null(None)),
            f("recorded_event_id", record_event_or_null(None)),
            f("matches_current_reference", no()),
            f("schema", s("raios.module_service_slot_reservation.v0")),
            f("status", s("missing")),
            f("reason", s("no_valid_service_slot_reservation_retained")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ]
    };
    emit_record_property_line("retained_service_slot_reservation", fields, comma);
}

#[rustfmt::skip]
fn emit_module_service_slot_policy_result(check: &ModuleServiceSlotReservationCheck<'_>, comma: bool) {
    emit_record_property_line(
        "policy_result",
        vec![
            f("reservation_reference_present", b(check.valid)),
            f("service_slot_reserved", no()),
            f("allocates_service_slot", no()),
            f("loader", s("unavailable")),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        comma,
    );
}

#[rustfmt::skip]
pub(crate) fn emit_module_service_slot_diagnostic_selftest() {
    let cases = module_service_slot_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }
    let case_records = cases.iter().map(module_service_slot_selftest_case_record).collect();

    begin_response("module.service_slot_diagnostic_selftest");
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_service_slot_reservation_diagnostic_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_service_slot_reservation_records", no()),
            f("allocates_service_slot", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f("cases", V::Array(case_records)),
        ],
        6,
    );
    emit_record_value_property_line("can_load", no(), false);
    end_response("module.service_slot_diagnostic_selftest");
}

#[rustfmt::skip]
fn module_service_slot_selftest_case_record(case: &ModuleServiceSlotSelfTestCase) -> V<'static> {
    V::InlineObject(vec![f("case", s(case.name)), f("expected_status", s(case.expected_status)), f("expected_reason", s(case.expected_reason)), f("actual_status", s(case.actual_status)), f("actual_reason", s(case.actual_reason)), f("passed", b(case.passed)), f("can_load", no()), f("load_attempted", no())])
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
    let (_, retained_reference) = event_log::latest_module_computed_grant_reference()?;
    let (latest_retained_event_id, _) = event_log::latest_module_computed_grant_reference()?;
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
