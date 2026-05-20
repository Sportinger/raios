use crate::{
    agent_protocol_module_audit::module_audit_rollback_valid_input,
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, crlf, current_boot_event_id_str, end_response, json_event_id,
        json_event_id_option, json_opt_str, json_sha256, json_sha256_option, json_str, method_eq,
        method_head_eq, parse_current_boot_event_id, parse_sha256_ref, raw, raw_bool, raw_fmt,
        raw_line,
    },
    event_log,
    module_evidence::{
        self, ram_only_service_slot_id_valid, ModuleServiceSlotReservationHashInput,
    },
};
pub(crate) fn module_service_slot_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "module.service_slot_diagnostic")
}

pub(crate) fn module_service_slot_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.service_slot_diagnostic_selftest")
}

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

    begin_response("module.service_slot_diagnostic");
    raw_line("      \"schema\": \"raios.module_service_slot_reservation_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"reference_format\": \"module.service_slot_diagnostic <reservation_hash> <retained_reference_event_id> <retained_audit_rollback_reference_event_id> <computed_grant_hash> <audit_record_hash> <rollback_plan_hash> <pre_load_service_inventory_hash> <ram_only_service_slot_id> [current_boot]\",");
    emit_module_service_slot_reference_object(&check);
    raw_line(",");
    emit_module_service_slot_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    emit_module_service_slot_policy_result(&check);
    raw_line(",");
    raw_line("      \"blocked_by\": [");
    raw_line(
        "        {\"gate\": \"service_slot_allocator\", \"state\": \"unavailable\", \"reason\": \"ram_only_service_slot_allocator_unimplemented\"},",
    );
    raw_line(
        "        {\"gate\": \"module_loader\", \"state\": \"unavailable\", \"reason\": \"module_loader_unimplemented\"}",
    );
    raw_line("      ]");
    end_response("module.service_slot_diagnostic");
}

fn emit_module_service_slot_reference_object(check: &ModuleServiceSlotReservationCheck<'_>) {
    raw_line("      \"service_slot_reservation_reference\": {");
    raw("        \"state\": ");
    json_str(if check.has_reference {
        "present"
    } else {
        "absent"
    });
    raw_line(",");
    raw("        \"validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw("        \"reservation_hash\": ");
    json_sha256_option(check.reservation_hash);
    raw_line(",");
    raw("        \"expected_reservation_hash\": ");
    json_sha256_option(check.expected_reservation_hash);
    raw_line(",");
    raw("        \"retained_computed_grant_reference_event_id\": ");
    json_opt_str(check.retained_reference_event_id);
    raw_line(",");
    raw("        \"retained_audit_rollback_reference_event_id\": ");
    json_opt_str(check.retained_audit_rollback_reference_event_id);
    raw_line(",");
    raw("        \"computed_capability_grant_hash\": ");
    json_sha256_option(check.computed_grant_hash);
    raw_line(",");
    raw("        \"audit_record_hash\": ");
    json_sha256_option(check.audit_record_hash);
    raw_line(",");
    raw("        \"rollback_plan_hash\": ");
    json_sha256_option(check.rollback_plan_hash);
    raw_line(",");
    raw("        \"pre_load_service_inventory_hash\": ");
    json_sha256_option(check.pre_load_service_inventory_hash);
    raw_line(",");
    raw("        \"ram_only_service_slot_id\": ");
    json_opt_str(check.ram_only_service_slot_id);
    crlf();
    raw_line("      }");
}

fn emit_module_service_slot_retained_reference(
    check: &ModuleServiceSlotReservationCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(event_log::EventId, event_log::ModuleServiceSlotReservation)>,
) {
    raw_line("      \"retained_service_slot_reservation\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw("        \"recorded_event_id\": ");
        json_event_id_option(recorded_event_id);
        raw_line(",");
        raw("        \"matches_current_reference\": ");
        raw_bool(module_service_slot_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.module_service_slot_reservation.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"allocates_service_slot\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw_line(",");
        raw("        \"retained_audit_rollback_reference_event_id\": ");
        json_event_id(reference.retained_audit_rollback_reference_event_id);
        raw_line(",");
        raw("        \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"reservation_hash\": ");
        json_sha256(reference.reservation_hash);
        raw_line(",");
        raw("          \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("          \"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw_line(",");
        raw("          \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw_line(",");
        raw("          \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.module_service_slot_reservation.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_service_slot_reservation_retained\",");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_module_service_slot_policy_result(check: &ModuleServiceSlotReservationCheck<'_>) {
    raw_line("      \"policy_result\": {");
    raw("        \"reservation_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"service_slot_reserved\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"loader\": \"unavailable\",");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw("      }");
}

pub(crate) fn emit_module_service_slot_diagnostic_selftest() {
    let cases = module_service_slot_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.service_slot_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.module_service_slot_reservation_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_service_slot_reservation_records\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_service_slot_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.service_slot_diagnostic_selftest");
}

fn emit_module_service_slot_selftest_case(case: &ModuleServiceSlotSelfTestCase, comma: bool) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
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
    let valid = module_service_slot_valid_input();
    [
        module_service_slot_selftest_case(
            "absent_reference",
            "missing",
            "service_slot_reservation_reference_absent",
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
        ),
        module_service_slot_selftest_case(
            "accepted_current_boot_reservation_still_denied",
            "valid_hash_reference_load_still_denied",
            "service_slot_reservation_valid_but_allocator_and_loader_missing",
            valid,
        ),
        module_service_slot_selftest_case(
            "stale_previous_boot_reservation",
            "stale_or_non_current_boot_reference",
            "service_slot_reservation_scope_must_be_current_boot",
            ModuleServiceSlotReservationInput {
                scope: "previous_boot",
                ..valid
            },
        ),
        module_service_slot_selftest_case(
            "mismatched_reservation_hash",
            "mismatched_reservation_hash",
            "service_slot_reservation_hash_mismatch",
            ModuleServiceSlotReservationInput {
                reservation_hash: Some([0x99; 32]),
                ..valid
            },
        ),
        module_service_slot_selftest_case(
            "invalid_ram_only_service_slot",
            "rejected",
            "ram_only_service_slot_id_invalid",
            ModuleServiceSlotReservationInput {
                ram_only_service_slot_id: Some("svc.test.0001"),
                ..valid
            },
        ),
    ]
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

fn module_service_slot_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleServiceSlotReservationInput<'_>,
) -> ModuleServiceSlotSelfTestCase {
    let actual = evaluate_module_service_slot_reservation(candidate, false);
    ModuleServiceSlotSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && actual.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

fn computed_module_service_slot_reservation_hash(
    input: ModuleServiceSlotReservationHashInput<'_>,
) -> [u8; 32] {
    module_evidence::computed_module_service_slot_reservation_hash(input)
}
