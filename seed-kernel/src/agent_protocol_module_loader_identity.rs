use crate::{
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, crlf, end_response, json_event_id_option, json_str, method_eq,
        method_head_eq, raw, raw_bool, raw_fmt, raw_line,
    },
    event_log,
};

pub(crate) fn module_loader_identity_method(method: &str) -> bool {
    method_head_eq(method, "module.loader_identity")
}

pub(crate) fn module_loader_identity_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.loader_identity_selftest")
}

pub(crate) fn emit_module_loader_identity() {
    let manifest = event_log::latest_module_manifest_reference();
    let artifact = event_log::latest_module_candidate_artifact_reference();
    let vm_report = event_log::latest_module_vm_test_report_reference();
    let local_attestation = event_log::latest_module_local_attestation_reference();
    let local_approval = event_log::latest_module_local_approval_reference();
    let computed_grant = event_log::latest_module_computed_grant_reference();
    let audit_rollback = event_log::latest_module_audit_rollback_reference();
    let service_slot = event_log::latest_module_service_slot_reservation();
    let retained_module_evidence_present = manifest.is_some()
        && artifact.is_some()
        && vm_report.is_some()
        && local_attestation.is_some()
        && local_approval.is_some()
        && computed_grant.is_some()
        && audit_rollback.is_some()
        && service_slot.is_some();
    let candidate = ModuleLoaderIdentityCandidate {
        retained_module_evidence_present,
        service_slot_allocator_readiness_present: true,
        service_slot_allocator_ready: false,
        audit_rollback_write_boundary_present: false,
        identity: module_loader_identity_missing_fact(),
    };
    let evaluation = evaluate_module_loader_identity_candidate(candidate);
    let source_evidence = module_loader_identity_source_evidence(
        candidate,
        evaluation,
        manifest.as_ref().map(|(event_id, _)| *event_id),
        artifact.as_ref().map(|(event_id, _)| *event_id),
        vm_report.as_ref().map(|(event_id, _)| *event_id),
        local_attestation.as_ref().map(|(event_id, _)| *event_id),
        local_approval.as_ref().map(|(event_id, _)| *event_id),
        computed_grant.as_ref().map(|(event_id, _)| *event_id),
        audit_rollback.as_ref().map(|(event_id, _)| *event_id),
        service_slot.as_ref().map(|(event_id, _)| *event_id),
    );
    let source_evidence_event_id =
        event_log::record_module_loader_identity_source_evidence(source_evidence);

    begin_response("module.loader_identity");
    raw_line("      \"schema\": \"raios.module_loader_identity.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": true,");
    raw_line(
        "      \"global_event_log_mutation\": \"retained_current_boot_source_evidence_only\",",
    );
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"can_load_now\": false,");
    raw_line("      \"load_attempted\": false,");
    emit_module_loader_identity_retained_module_evidence(
        retained_module_evidence_present,
        manifest.as_ref().map(|(event_id, _)| *event_id),
        artifact.as_ref().map(|(event_id, _)| *event_id),
        vm_report.as_ref().map(|(event_id, _)| *event_id),
        local_attestation.as_ref().map(|(event_id, _)| *event_id),
        local_approval.as_ref().map(|(event_id, _)| *event_id),
        computed_grant.as_ref().map(|(event_id, _)| *event_id),
        audit_rollback.as_ref().map(|(event_id, _)| *event_id),
        service_slot.as_ref().map(|(event_id, _)| *event_id),
    );
    raw_line(",");
    emit_module_loader_identity_source_evidence(source_evidence_event_id, source_evidence);
    raw_line(",");
    emit_module_loader_identity_required_bindings(candidate, evaluation);
    raw_line(",");
    emit_module_loader_identity_fact(candidate.identity, evaluation, source_evidence_event_id);
    raw_line(",");
    emit_module_loader_identity_policy_result(candidate, evaluation);
    raw_line(",");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    emit_module_loader_identity_gate(
        &mut wrote,
        "retained_module_evidence",
        evaluation.retained_module_evidence_status,
        evaluation.retained_module_evidence_reason,
    );
    emit_module_loader_identity_gate(
        &mut wrote,
        "service_slot_allocator_readiness",
        evaluation.service_slot_allocator_readiness_status,
        evaluation.service_slot_allocator_readiness_reason,
    );
    emit_module_loader_identity_gate(
        &mut wrote,
        "service_slot_allocator_runtime",
        evaluation.service_slot_allocator_runtime_status,
        evaluation.service_slot_allocator_runtime_reason,
    );
    emit_module_loader_identity_gate(
        &mut wrote,
        "audit_rollback_write_boundary",
        evaluation.audit_rollback_write_boundary_status,
        evaluation.audit_rollback_write_boundary_reason,
    );
    emit_module_loader_identity_gate(
        &mut wrote,
        "loader_identity",
        evaluation.identity_status,
        evaluation.identity_reason,
    );
    crlf();
    raw_line("      ]");
    end_response("module.loader_identity");
}

pub(crate) fn emit_module_loader_identity_selftest() {
    let cases = module_loader_identity_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.loader_identity_selftest");
    raw_line("      \"schema\": \"raios.module_loader_identity_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"can_load_now\": false,");
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
        emit_module_loader_identity_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.loader_identity_selftest");
}

fn emit_module_loader_identity_retained_module_evidence(
    retained_module_evidence_present: bool,
    manifest_event_id: Option<event_log::EventId>,
    artifact_event_id: Option<event_log::EventId>,
    vm_report_event_id: Option<event_log::EventId>,
    local_attestation_event_id: Option<event_log::EventId>,
    local_approval_event_id: Option<event_log::EventId>,
    computed_grant_event_id: Option<event_log::EventId>,
    audit_rollback_event_id: Option<event_log::EventId>,
    service_slot_event_id: Option<event_log::EventId>,
) {
    raw_line("      \"retained_module_evidence\": {");
    raw("        \"state\": ");
    json_str(if retained_module_evidence_present {
        "available"
    } else {
        "missing"
    });
    raw_line(",");
    raw("        \"status\": ");
    json_str(if retained_module_evidence_present {
        "available"
    } else {
        "missing"
    });
    raw_line(",");
    raw("        \"reason\": ");
    json_str(if retained_module_evidence_present {
        "retained_module_evidence_available"
    } else {
        "retained_module_evidence_missing"
    });
    raw_line(",");
    raw_line("        \"classification\": \"local_only\",");
    raw_line("        \"authority\": \"current_boot_hash_references\",");
    raw("        \"manifest_reference_event_id\": ");
    json_event_id_option(manifest_event_id);
    raw_line(",");
    raw("        \"candidate_artifact_reference_event_id\": ");
    json_event_id_option(artifact_event_id);
    raw_line(",");
    raw("        \"vm_test_report_reference_event_id\": ");
    json_event_id_option(vm_report_event_id);
    raw_line(",");
    raw("        \"local_attestation_reference_event_id\": ");
    json_event_id_option(local_attestation_event_id);
    raw_line(",");
    raw("        \"local_approval_reference_event_id\": ");
    json_event_id_option(local_approval_event_id);
    raw_line(",");
    raw("        \"computed_grant_reference_event_id\": ");
    json_event_id_option(computed_grant_event_id);
    raw_line(",");
    raw("        \"audit_rollback_reference_event_id\": ");
    json_event_id_option(audit_rollback_event_id);
    raw_line(",");
    raw("        \"service_slot_reservation_event_id\": ");
    json_event_id_option(service_slot_event_id);
    crlf();
    raw_line("      }");
}

fn emit_module_loader_identity_source_evidence(
    event_id: event_log::EventId,
    evidence: event_log::ModuleLoaderIdentitySourceEvidence,
) {
    raw_line("      \"source_evidence\": {");
    raw("        \"schema\": ");
    json_str(evidence.schema);
    raw_line(",");
    raw_line("        \"state\": \"retained\",");
    raw_line("        \"status\": \"retained_current_boot_source_evidence\",");
    raw_line("        \"reason\": \"module_loader_identity_source_evidence_recorded\",");
    raw_line("        \"scope\": \"current_boot\",");
    raw_line("        \"classification\": \"local_only\",");
    raw_line("        \"retention\": \"current_boot_ram_event_log\",");
    raw("        \"event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("        \"fact_schema\": ");
    json_str(evidence.fact_schema);
    raw_line(",");
    raw("        \"fact_id\": ");
    json_str(evidence.fact_id);
    raw_line(",");
    raw("        \"source_method\": ");
    json_str(evidence.source_method);
    raw_line(",");
    raw("        \"source_fact_locator\": ");
    json_str(evidence.source_fact_locator);
    raw_line(",");
    raw("        \"readiness_status\": ");
    json_str(evidence.readiness_status);
    raw_line(",");
    raw("        \"readiness_reason\": ");
    json_str(evidence.readiness_reason);
    raw_line(",");
    raw("        \"identity_status\": ");
    json_str(evidence.identity_status);
    raw_line(",");
    raw("        \"identity_reason\": ");
    json_str(evidence.identity_reason);
    raw_line(",");
    raw("        \"identity_present\": ");
    raw_bool(evidence.identity_present);
    raw_line(",");
    raw("        \"retained_module_evidence_present\": ");
    raw_bool(evidence.retained_module_evidence_present);
    raw_line(",");
    raw("        \"service_slot_allocator_readiness_present\": ");
    raw_bool(evidence.service_slot_allocator_readiness_present);
    raw_line(",");
    raw("        \"service_slot_allocator_ready\": ");
    raw_bool(evidence.service_slot_allocator_ready);
    raw_line(",");
    raw("        \"audit_rollback_write_boundary_present\": ");
    raw_bool(evidence.audit_rollback_write_boundary_present);
    raw_line(",");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false,");
    raw_line("        \"authorizes_load\": false");
    raw_line("      }");
}

fn emit_module_loader_identity_required_bindings(
    candidate: ModuleLoaderIdentityCandidate,
    evaluation: ModuleLoaderIdentityEvaluation,
) {
    raw_line("      \"required_bindings\": {");
    raw("        \"retained_module_evidence\": ");
    json_str(evaluation.retained_module_evidence_status);
    raw_line(",");
    raw("        \"service_slot_allocator_readiness\": ");
    json_str(evaluation.service_slot_allocator_readiness_status);
    raw_line(",");
    raw("        \"service_slot_allocator_runtime\": ");
    json_str(evaluation.service_slot_allocator_runtime_status);
    raw_line(",");
    raw("        \"audit_rollback_write_boundary\": ");
    json_str(evaluation.audit_rollback_write_boundary_status);
    raw_line(",");
    raw("        \"loader_identity_fact_present\": ");
    raw_bool(candidate.identity.present);
    crlf();
    raw_line("      }");
}

fn emit_module_loader_identity_fact(
    fact: ModuleLoaderRuntimeFact,
    evaluation: ModuleLoaderIdentityEvaluation,
    source_evidence_event_id: event_log::EventId,
) {
    raw_line("      \"loader_identity\": {");
    raw_line("        \"schema\": \"raios.module_loader_identity.v0\",");
    raw("        \"state\": ");
    json_str(if fact.present { "present" } else { "missing" });
    raw_line(",");
    raw("        \"status\": ");
    json_str(evaluation.identity_status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(evaluation.identity_reason);
    raw_line(",");
    raw_line("        \"scope\": \"current_boot\",");
    raw("        \"fact_scope\": ");
    json_str(fact.scope);
    raw_line(",");
    raw("        \"schema_valid\": ");
    raw_bool(fact.schema_ok);
    raw_line(",");
    raw("        \"classification\": ");
    json_str(fact.classification);
    raw_line(",");
    raw("        \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw_line(",");
    raw("        \"binds_retained_module_evidence\": ");
    raw_bool(fact.binds_retained_module_evidence);
    raw_line(",");
    raw("        \"binds_service_slot_allocator\": ");
    raw_bool(fact.binds_service_slot_allocator);
    raw_line(",");
    raw("        \"binds_audit_rollback_write_boundary\": ");
    raw_bool(fact.binds_audit_rollback_write_boundary);
    raw_line(",");
    raw_line("        \"fact_id\": \"module.loader_runtime.identity.current_boot\",");
    raw_line("        \"source_method\": \"module.loader_identity\",");
    raw_line("        \"source_fact_locator\": \"module.loader_identity.loader_identity\",");
    raw("        \"source_evidence_event_id\": ");
    json_event_id_option(Some(source_evidence_event_id));
    raw_line(",");
    raw_line(
        "        \"source_evidence_schema\": \"raios.module_loader_identity_source_evidence.v0\",",
    );
    raw_line("        \"source_evidence_state\": \"retained_current_boot\",");
    raw_line("        \"persistence\": \"none\",");
    raw_line("        \"durable\": false,");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"authorizes_load\": false");
    raw_line("      }");
}

fn emit_module_loader_identity_policy_result(
    candidate: ModuleLoaderIdentityCandidate,
    evaluation: ModuleLoaderIdentityEvaluation,
) {
    raw_line("      \"policy_result\": {");
    raw("        \"readiness_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"readiness_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"retained_module_evidence_present\": ");
    raw_bool(candidate.retained_module_evidence_present);
    raw_line(",");
    raw("        \"service_slot_allocator_readiness_present\": ");
    raw_bool(candidate.service_slot_allocator_readiness_present);
    raw_line(",");
    raw("        \"service_slot_allocator_ready\": ");
    raw_bool(candidate.service_slot_allocator_ready);
    raw_line(",");
    raw("        \"audit_rollback_write_boundary_present\": ");
    raw_bool(candidate.audit_rollback_write_boundary_present);
    raw_line(",");
    raw("        \"identity_available\": ");
    raw_bool(method_eq(evaluation.identity_status, "available"));
    raw_line(",");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
}

fn emit_module_loader_identity_gate(
    wrote: &mut bool,
    gate: &'static str,
    state: &'static str,
    reason: &'static str,
) {
    if method_eq(state, "available") {
        return;
    }
    if *wrote {
        raw_line(",");
    } else {
        *wrote = true;
    }
    raw("        {\"gate\": ");
    json_str(gate);
    raw(", \"state\": ");
    json_str(state);
    raw(", \"reason\": ");
    json_str(reason);
    raw("}");
}

fn emit_module_loader_identity_selftest_case(case: &ModuleLoaderIdentitySelfTestCase, comma: bool) {
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
    raw(", \"actual_identity_status\": ");
    json_str(case.actual_identity_status);
    raw(", \"actual_identity_reason\": ");
    json_str(case.actual_identity_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"loads_artifact\": false, \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn module_loader_identity_source_evidence(
    candidate: ModuleLoaderIdentityCandidate,
    evaluation: ModuleLoaderIdentityEvaluation,
    manifest_event_id: Option<event_log::EventId>,
    artifact_event_id: Option<event_log::EventId>,
    vm_report_event_id: Option<event_log::EventId>,
    local_attestation_event_id: Option<event_log::EventId>,
    local_approval_event_id: Option<event_log::EventId>,
    computed_grant_event_id: Option<event_log::EventId>,
    audit_rollback_event_id: Option<event_log::EventId>,
    service_slot_event_id: Option<event_log::EventId>,
) -> event_log::ModuleLoaderIdentitySourceEvidence {
    event_log::ModuleLoaderIdentitySourceEvidence {
        schema: "raios.module_loader_identity_source_evidence.v0",
        fact_schema: "raios.module_loader_identity.v0",
        fact_id: "module.loader_runtime.identity.current_boot",
        source_method: "module.loader_identity",
        source_fact_locator: "module.loader_identity.loader_identity",
        readiness_status: evaluation.status,
        readiness_reason: evaluation.reason,
        identity_status: evaluation.identity_status,
        identity_reason: evaluation.identity_reason,
        identity_present: candidate.identity.present,
        identity_scope: candidate.identity.scope,
        identity_schema_ok: candidate.identity.schema_ok,
        identity_provenance_ok: candidate.identity.provenance_ok,
        identity_classification: candidate.identity.classification,
        retained_module_evidence_present: candidate.retained_module_evidence_present,
        service_slot_allocator_readiness_present: candidate
            .service_slot_allocator_readiness_present,
        service_slot_allocator_ready: candidate.service_slot_allocator_ready,
        audit_rollback_write_boundary_present: candidate.audit_rollback_write_boundary_present,
        binds_retained_module_evidence: candidate.identity.binds_retained_module_evidence,
        binds_service_slot_allocator: candidate.identity.binds_service_slot_allocator,
        binds_audit_rollback_write_boundary: candidate.identity.binds_audit_rollback_write_boundary,
        manifest_reference_event_id: manifest_event_id,
        artifact_reference_event_id: artifact_event_id,
        vm_test_report_reference_event_id: vm_report_event_id,
        local_attestation_reference_event_id: local_attestation_event_id,
        local_approval_reference_event_id: local_approval_event_id,
        computed_grant_reference_event_id: computed_grant_event_id,
        audit_rollback_reference_event_id: audit_rollback_event_id,
        service_slot_reservation_event_id: service_slot_event_id,
    }
}

fn evaluate_module_loader_identity_candidate(
    candidate: ModuleLoaderIdentityCandidate,
) -> ModuleLoaderIdentityEvaluation {
    let (retained_module_evidence_status, retained_module_evidence_reason) =
        if candidate.retained_module_evidence_present {
            ("available", "retained_module_evidence_available")
        } else {
            ("missing", "retained_module_evidence_missing")
        };
    let (service_slot_allocator_readiness_status, service_slot_allocator_readiness_reason) =
        if candidate.service_slot_allocator_readiness_present {
            ("available", "service_slot_allocator_readiness_available")
        } else {
            ("missing", "service_slot_allocator_readiness_missing")
        };
    let (service_slot_allocator_runtime_status, service_slot_allocator_runtime_reason) =
        if candidate.service_slot_allocator_ready {
            ("available", "service_slot_allocator_runtime_available")
        } else {
            ("missing", "service_slot_allocator_runtime_missing")
        };
    let (audit_rollback_write_boundary_status, audit_rollback_write_boundary_reason) =
        if candidate.audit_rollback_write_boundary_present {
            (
                "available",
                "module_audit_rollback_write_boundary_binding_available",
            )
        } else {
            (
                "missing",
                "module_audit_rollback_write_boundary_binding_missing",
            )
        };
    let (identity_status, identity_reason) =
        evaluate_module_loader_identity_fact(candidate.identity);

    let (status, reason) = if !candidate.retained_module_evidence_present {
        (
            "denied_missing_retained_module_evidence",
            retained_module_evidence_reason,
        )
    } else if !candidate.service_slot_allocator_readiness_present {
        (
            "denied_missing_service_slot_allocator_readiness",
            service_slot_allocator_readiness_reason,
        )
    } else if !candidate.service_slot_allocator_ready {
        (
            "denied_missing_service_slot_allocator_runtime",
            service_slot_allocator_runtime_reason,
        )
    } else if !candidate.audit_rollback_write_boundary_present {
        (
            "denied_missing_audit_rollback_write_boundary",
            audit_rollback_write_boundary_reason,
        )
    } else if method_eq(identity_status, "rejected") {
        ("rejected", identity_reason)
    } else if method_eq(identity_status, "missing") {
        ("denied_missing_loader_identity", identity_reason)
    } else {
        (
            "available_non_authorizing",
            "module_loader_identity_not_load_authority",
        )
    };

    ModuleLoaderIdentityEvaluation {
        status,
        reason,
        retained_module_evidence_status,
        retained_module_evidence_reason,
        service_slot_allocator_readiness_status,
        service_slot_allocator_readiness_reason,
        service_slot_allocator_runtime_status,
        service_slot_allocator_runtime_reason,
        audit_rollback_write_boundary_status,
        audit_rollback_write_boundary_reason,
        identity_status,
        identity_reason,
        loads_artifact: false,
        allocates_service_slot: false,
        creates_service_inventory_records: false,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_loader_identity_fact(
    fact: ModuleLoaderRuntimeFact,
) -> (&'static str, &'static str) {
    if !method_eq(fact.scope, "current_boot") {
        return (
            "rejected",
            "module_loader_identity_scope_must_be_current_boot",
        );
    }
    if !fact.schema_ok {
        return ("rejected", "module_loader_identity_schema_mismatch");
    }
    if !fact.present {
        return ("missing", "module_loader_identity_missing");
    }
    if !fact.provenance_ok {
        return ("rejected", "module_loader_identity_provenance_missing");
    }
    if !fact.binds_retained_module_evidence {
        return (
            "rejected",
            "module_loader_identity_retained_evidence_binding_missing",
        );
    }
    if !fact.binds_service_slot_allocator {
        return (
            "rejected",
            "module_loader_identity_service_slot_allocator_binding_missing",
        );
    }
    if !fact.binds_audit_rollback_write_boundary {
        return (
            "rejected",
            "module_loader_identity_audit_write_boundary_binding_missing",
        );
    }
    ("available", "module_loader_identity_available")
}

fn module_loader_identity_selftest_cases(
) -> [ModuleLoaderIdentitySelfTestCase; MODULE_LOADER_IDENTITY_SELFTEST_CASES] {
    let ready = module_loader_identity_ready_candidate();
    let missing_fact = module_loader_identity_missing_fact();
    [
        module_loader_identity_selftest_case(
            "missing_retained_module_evidence",
            "denied_missing_retained_module_evidence",
            "retained_module_evidence_missing",
            ModuleLoaderIdentityCandidate {
                retained_module_evidence_present: false,
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "missing_service_slot_allocator_readiness",
            "denied_missing_service_slot_allocator_readiness",
            "service_slot_allocator_readiness_missing",
            ModuleLoaderIdentityCandidate {
                service_slot_allocator_readiness_present: false,
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "service_slot_allocator_runtime_missing",
            "denied_missing_service_slot_allocator_runtime",
            "service_slot_allocator_runtime_missing",
            ModuleLoaderIdentityCandidate {
                service_slot_allocator_ready: false,
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "audit_write_boundary_missing",
            "denied_missing_audit_rollback_write_boundary",
            "module_audit_rollback_write_boundary_binding_missing",
            ModuleLoaderIdentityCandidate {
                audit_rollback_write_boundary_present: false,
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "loader_identity_previous_boot",
            "rejected",
            "module_loader_identity_scope_must_be_current_boot",
            ModuleLoaderIdentityCandidate {
                identity: ModuleLoaderRuntimeFact {
                    scope: "previous_boot",
                    ..ready.identity
                },
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "loader_identity_schema_mismatch",
            "rejected",
            "module_loader_identity_schema_mismatch",
            ModuleLoaderIdentityCandidate {
                identity: ModuleLoaderRuntimeFact {
                    schema_ok: false,
                    ..ready.identity
                },
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "loader_identity_provenance_missing",
            "rejected",
            "module_loader_identity_provenance_missing",
            ModuleLoaderIdentityCandidate {
                identity: ModuleLoaderRuntimeFact {
                    provenance_ok: false,
                    ..ready.identity
                },
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "loader_identity_retained_evidence_binding_missing",
            "rejected",
            "module_loader_identity_retained_evidence_binding_missing",
            ModuleLoaderIdentityCandidate {
                identity: ModuleLoaderRuntimeFact {
                    binds_retained_module_evidence: false,
                    ..ready.identity
                },
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "loader_identity_service_slot_allocator_binding_missing",
            "rejected",
            "module_loader_identity_service_slot_allocator_binding_missing",
            ModuleLoaderIdentityCandidate {
                identity: ModuleLoaderRuntimeFact {
                    binds_service_slot_allocator: false,
                    ..ready.identity
                },
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "loader_identity_audit_write_boundary_binding_missing",
            "rejected",
            "module_loader_identity_audit_write_boundary_binding_missing",
            ModuleLoaderIdentityCandidate {
                identity: ModuleLoaderRuntimeFact {
                    binds_audit_rollback_write_boundary: false,
                    ..ready.identity
                },
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "loader_identity_missing",
            "denied_missing_loader_identity",
            "module_loader_identity_missing",
            ModuleLoaderIdentityCandidate {
                identity: missing_fact,
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "all_inputs_present_identity_non_authorizing",
            "available_non_authorizing",
            "module_loader_identity_not_load_authority",
            ready,
        ),
    ]
}

fn module_loader_identity_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoaderIdentityCandidate,
) -> ModuleLoaderIdentitySelfTestCase {
    let actual = evaluate_module_loader_identity_candidate(candidate);
    ModuleLoaderIdentitySelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_identity_status: actual.identity_status,
        actual_identity_reason: actual.identity_reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.loads_artifact
            && !actual.allocates_service_slot
            && !actual.creates_service_inventory_records
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_loader_identity_ready_candidate() -> ModuleLoaderIdentityCandidate {
    ModuleLoaderIdentityCandidate {
        retained_module_evidence_present: true,
        service_slot_allocator_readiness_present: true,
        service_slot_allocator_ready: true,
        audit_rollback_write_boundary_present: true,
        identity: module_loader_identity_available_fact(),
    }
}

fn module_loader_identity_missing_fact() -> ModuleLoaderRuntimeFact {
    ModuleLoaderRuntimeFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_retained_module_evidence: false,
        binds_service_slot_allocator: false,
        binds_audit_rollback_write_boundary: false,
        source_evidence_event_id: None,
        source_evidence_schema: "raios.module_loader_identity_source_evidence.v0",
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: "missing",
        source_evidence_reason: "module_loader_identity_source_evidence_missing",
        source_evidence_method: "module.loader_identity",
        source_evidence_fact_locator: "module.loader_identity.loader_identity",
    }
}

fn module_loader_identity_available_fact() -> ModuleLoaderRuntimeFact {
    ModuleLoaderRuntimeFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_retained_module_evidence: true,
        binds_service_slot_allocator: true,
        binds_audit_rollback_write_boundary: true,
        source_evidence_event_id: None,
        source_evidence_schema: "raios.module_loader_identity_source_evidence.v0",
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: "available",
        source_evidence_reason: "module_loader_identity_available",
        source_evidence_method: "module.loader_identity",
        source_evidence_fact_locator: "module.loader_identity.loader_identity",
    }
}
