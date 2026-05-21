use crate::{
    agent_protocol_support::{
        begin_response, crlf, current_boot_event_id_str, end_response, json_event_id,
        json_event_id_option, json_opt_str, json_sha256, json_sha256_option, json_str, method_eq,
        method_head_eq, parse_current_boot_event_id, parse_sha256_ref, raw, raw_bool, raw_fmt,
        raw_line,
    },
    event_log, module_evidence, serial,
};

pub(crate) const RECOVERY_ARTIFACT_LOAD_METHOD: &str = "recovery.load_artifact";
pub(crate) const MODULE_RECOVERY_ARTIFACT_LOAD_METHOD: &str = "module.load_recovery_artifact";
pub(crate) const RECOVERY_ARTIFACT_LOAD_CAPABILITY: &str = "cap.recovery.load_artifact";
pub(crate) const RECOVERY_ARTIFACT_LOAD_READ_CAPABILITY: &str = "cap.recovery.load_artifact.read";
pub(crate) const RECOVERY_ARTIFACT_LOAD_BINDING_METHOD: &str = "recovery.load_binding";
pub(crate) const MODULE_RECOVERY_ARTIFACT_LOAD_BINDING_METHOD: &str =
    "module.recovery_load_binding";
pub(crate) const RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD: &str =
    "recovery.load_binding_selftest";
pub(crate) const MODULE_RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD: &str =
    "module.recovery_load_binding_selftest";
const RECOVERY_LOAD_BINDING_SELFTEST_CASES: usize = 14;
const RECOVERY_IDENTITY_SELFTEST_CASES: usize = 6;
const RECOVERY_TRUST_SELFTEST_CASES: usize = 8;

#[derive(Clone, Copy)]
struct RecoveryIdentityReferenceCheck<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    identity_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    expected_identity_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
}

struct RecoveryIdentitySelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

#[derive(Clone, Copy)]
struct RecoveryTrustReferenceInput<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    trust_reference_hash: Option<[u8; 32]>,
    retained_identity_reference_event_id: Option<&'a str>,
    identity_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    trust_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
struct RecoveryTrustReferenceCheck<'a> {
    has_reference: bool,
    arity_valid: bool,
    scope: &'a str,
    trust_reference_hash: Option<[u8; 32]>,
    retained_identity_reference_event_id: Option<&'a str>,
    identity_reference_hash: Option<[u8; 32]>,
    artifact_hash: Option<[u8; 32]>,
    trust_hash: Option<[u8; 32]>,
    expected_trust_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
}

struct RecoveryTrustSelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

#[derive(Clone, Copy)]
struct RecoveryEvidenceCandidate {
    retained: bool,
    current_boot: bool,
    schema_ok: bool,
    binding_ok: bool,
    binding_reason: &'static str,
}

#[derive(Clone, Copy)]
struct RecoveryLoadBindingCandidate {
    requested_capability: &'static str,
    identity: RecoveryEvidenceCandidate,
    trust: RecoveryEvidenceCandidate,
    vm_test: RecoveryEvidenceCandidate,
    local_approval: RecoveryEvidenceCandidate,
    loader: RecoveryEvidenceCandidate,
    rollback_evidence: RecoveryEvidenceCandidate,
    normal_module_capability_substituted: bool,
    normal_module_append_intent_substituted: bool,
    append_payload_hash_claimed_authority: bool,
    normal_module_writer_facts_substituted: bool,
    normal_module_service_slot_substituted: bool,
}

#[derive(Clone, Copy)]
struct RecoveryLoadBindingCheck {
    status: &'static str,
    reason: &'static str,
    recovery_only_capability_used: bool,
    accepts_normal_module_authority: bool,
    append_payload_hash_authority: bool,
    can_move_beyond_denial: bool,
    loads_recovery_artifact: bool,
    loads_normal_module: bool,
    creates_durable_records: bool,
    installs_rollback_plan: bool,
    service_inventory_change: &'static str,
    load_attempted: bool,
}

struct RecoveryLoadBindingSelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    passed: bool,
}

pub(crate) fn recovery_artifact_load_method(method: &str) -> bool {
    crate::agent_protocol_support::method_eq(method, RECOVERY_ARTIFACT_LOAD_METHOD)
        || crate::agent_protocol_support::method_eq(method, MODULE_RECOVERY_ARTIFACT_LOAD_METHOD)
}

pub(crate) fn recovery_artifact_identity_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.identity_diagnostic")
}

pub(crate) fn recovery_artifact_identity_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.identity_diagnostic_selftest")
}

pub(crate) fn recovery_artifact_trust_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.trust_diagnostic")
}

pub(crate) fn recovery_artifact_trust_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.trust_diagnostic_selftest")
}

pub(crate) fn recovery_artifact_load_binding_method(method: &str) -> bool {
    method_head_eq(method, RECOVERY_ARTIFACT_LOAD_BINDING_METHOD)
        || method_head_eq(method, MODULE_RECOVERY_ARTIFACT_LOAD_BINDING_METHOD)
}

pub(crate) fn recovery_artifact_load_binding_selftest_method(method: &str) -> bool {
    method_head_eq(method, RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD)
        || method_head_eq(
            method,
            MODULE_RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD,
        )
}

pub(crate) fn canonical_recovery_artifact_load_method(method: &str) -> &'static str {
    if crate::agent_protocol_support::method_eq(method, MODULE_RECOVERY_ARTIFACT_LOAD_METHOD) {
        MODULE_RECOVERY_ARTIFACT_LOAD_METHOD
    } else {
        RECOVERY_ARTIFACT_LOAD_METHOD
    }
}

pub(crate) fn emit_recovery_artifact_identity_diagnostic(method: &str) {
    let check = parse_recovery_identity_reference(recovery_identity_diagnostic_arg(method));
    let recorded_event_id = if check.valid {
        recovery_identity_binding_from_check(&check)
            .map(event_log::record_recovery_artifact_identity_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_artifact_identity_reference();

    begin_response("recovery.identity_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_artifact_identity_diagnostic.v0\",");
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
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.identity_diagnostic <identity_reference_hash> <artifact_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"identity_schema\": \"raios.recovery_artifact_identity.v0\",");
    raw_line(
        "        \"identity_canonicalization\": \"raios.recovery_artifact_identity.canonical.v0\"",
    );
    raw_line("      },");
    emit_recovery_identity_reference_object(&check);
    raw_line(",");
    emit_recovery_identity_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"identity_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.identity_diagnostic");
}

pub(crate) fn emit_recovery_artifact_identity_diagnostic_selftest() {
    let cases = recovery_identity_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.identity_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_artifact_identity_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_identity_records\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
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
        emit_recovery_identity_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.identity_diagnostic_selftest");
}

pub(crate) fn emit_recovery_artifact_trust_diagnostic(method: &str) {
    let check = parse_recovery_trust_reference(recovery_trust_diagnostic_arg(method), true);
    let recorded_event_id = if check.valid {
        recovery_trust_binding_from_check(&check)
            .map(event_log::record_recovery_artifact_trust_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_artifact_trust_reference();

    begin_response("recovery.trust_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_artifact_trust_diagnostic.v0\",");
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
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.trust_diagnostic <trust_reference_hash> <retained_identity_event_id> <identity_reference_hash> <artifact_hash> <trust_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"trust_schema\": \"raios.recovery_artifact_trust.v0\",");
    raw_line("        \"trust_canonicalization\": \"raios.recovery_artifact_trust.canonical.v0\"");
    raw_line("      },");
    emit_recovery_trust_reference_object(&check);
    raw_line(",");
    emit_recovery_trust_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"trust_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.trust_diagnostic");
}

pub(crate) fn emit_recovery_artifact_trust_diagnostic_selftest() {
    let cases = recovery_trust_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.trust_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_artifact_trust_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_trust_records\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
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
        emit_recovery_trust_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.trust_diagnostic_selftest");
}

pub(crate) fn emit_recovery_artifact_load_binding() {
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let live = evaluate_recovery_load_binding(recovery_load_binding_candidate_from_retained(
        retained_identity,
        retained_trust,
    ));

    begin_response(RECOVERY_ARTIFACT_LOAD_BINDING_METHOD);
    raw_line("      \"schema\": \"raios.recovery_artifact_load_binding.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"status\": \"denied_missing_recovery_binding\",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_records\": false,");
    raw_line("      \"request\": {");
    raw("        \"requested_capability\": ");
    json_str(RECOVERY_ARTIFACT_LOAD_CAPABILITY);
    raw_line(",");
    raw("        \"read_capability\": ");
    json_str(RECOVERY_ARTIFACT_LOAD_READ_CAPABILITY);
    raw_line(",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"risk\": \"recovery_modify_ram\",");
    raw_line("        \"target\": \"recovery_lifeline\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"recovery_only_capability_used\": true,");
    raw_line("        \"normal_module_capability_used\": false,");
    raw_line("        \"normal_module_load_path_used\": false,");
    raw_line("        \"separate_from\": \"cap.module.load_ephemeral\"");
    raw_line("      },");
    raw_line("      \"required_retained_evidence_ids\": [");
    raw_line("        \"recovery_artifact_identity_event_id\",");
    raw_line("        \"recovery_artifact_trust_event_id\",");
    raw_line("        \"recovery_vm_test_event_id\",");
    raw_line("        \"recovery_local_approval_event_id\",");
    raw_line("        \"recovery_loader_event_id\",");
    raw_line("        \"recovery_rollback_evidence_event_id\"");
    raw_line("      ],");
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_artifact_load_missing_fact(
        "recovery_vm_test_event_id",
        "raios.recovery_artifact_vm_test.v0",
        "recovery_vm_test_event_id_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_local_approval_event_id",
        "raios.recovery_artifact_local_approval.v0",
        "recovery_local_approval_event_id_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_loader_event_id",
        "raios.recovery_artifact_loader.v0",
        "recovery_loader_event_id_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_rollback_evidence_event_id",
        "raios.recovery_artifact_rollback_evidence.v0",
        "recovery_rollback_evidence_event_id_missing",
        false,
    );
    raw_line("      },");
    raw_line("      \"normal_module_authority_rejected\": {");
    raw_line("        \"module_load_ephemeral_facts_used\": false,");
    raw_line("        \"module_append_intent_used\": false,");
    raw_line("        \"module_append_payload_hash_used_as_authority\": false,");
    raw_line("        \"module_writer_facts_used\": false,");
    raw_line("        \"module_service_slot_used\": false,");
    raw_line("        \"normal_module_capability_accepted\": false");
    raw_line("      },");
    raw_line("      \"append_payload_hash_envelopes\": {");
    raw_line("        \"schema\": \"raios.module_audit_rollback_append_payload_hash.v0\",");
    raw_line("        \"authority\": false,");
    raw_line("        \"non_authority_input_only\": true,");
    raw_line("        \"append_payload_hash_authority\": false");
    raw_line("      },");
    raw_line("      \"boundary\": {");
    emit_recovery_load_binding_check(&live, 8, true);
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote_blocker = false;
    if retained_identity.is_none() {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_artifact_identity_event_id",
            "missing",
            "recovery_artifact_identity_event_id_missing",
        );
    }
    if retained_trust.is_none() {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_artifact_trust_event_id",
            "missing",
            "recovery_artifact_trust_event_id_missing",
        );
    } else if let Some(reason) =
        recovery_load_binding_retained_trust_mismatch(retained_identity, retained_trust)
    {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_artifact_trust_event_id",
            "rejected",
            reason,
        );
    }
    emit_recovery_load_blocker(
        &mut wrote_blocker,
        "recovery_vm_test_event_id",
        "missing",
        "recovery_vm_test_event_id_missing",
    );
    emit_recovery_load_blocker(
        &mut wrote_blocker,
        "recovery_local_approval_event_id",
        "missing",
        "recovery_local_approval_event_id_missing",
    );
    emit_recovery_load_blocker(
        &mut wrote_blocker,
        "recovery_loader_event_id",
        "missing",
        "recovery_loader_event_id_missing",
    );
    emit_recovery_load_blocker(
        &mut wrote_blocker,
        "recovery_rollback_evidence_event_id",
        "missing",
        "recovery_rollback_evidence_event_id_missing",
    );
    crlf();
    raw_line("      ]");
    end_response(RECOVERY_ARTIFACT_LOAD_BINDING_METHOD);
}

pub(crate) fn emit_recovery_artifact_load_binding_selftest() {
    let cases = recovery_load_binding_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response(RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD);
    raw_line("      \"schema\": \"raios.recovery_artifact_load_binding_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_records\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("      \"normal_module_capability_accepted\": false,");
    raw_line("      \"append_payload_hash_authority\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"required_retained_evidence_ids\": [");
    raw_line("        \"recovery_artifact_identity_event_id\",");
    raw_line("        \"recovery_artifact_trust_event_id\",");
    raw_line("        \"recovery_vm_test_event_id\",");
    raw_line("        \"recovery_local_approval_event_id\",");
    raw_line("        \"recovery_loader_event_id\",");
    raw_line("        \"recovery_rollback_evidence_event_id\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_load_binding_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response(RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD);
}

pub(crate) fn emit_recovery_artifact_load_denied(
    method: &'static str,
    event_id: event_log::EventId,
) {
    serial::write_raw_fmt(format_args!("RAIOS_AGENT_BEGIN {}\r\n", method));
    raw_line("{");
    raw_line("  \"v\": \"raios.agent.v0\",");
    raw_line("  \"t\": \"error\",");
    raw_line("  \"id\": \"serial\",");
    raw_line("  \"body\": {");
    raw("    \"method\": ");
    json_str(method);
    raw_line(",");
    raw("    \"event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("    \"audit_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw_line("    \"code\": \"capability_denied\",");
    raw_line("    \"schema\": \"raios.recovery_artifact_load_boundary.v0\",");
    raw("    \"message\": ");
    json_str("recovery artifact loading is denied until recovery-only identity, trust, VM-test, approval, loader, and rollback evidence exist");
    raw_line(",");
    raw_line("    \"request\": {");
    raw_line("      \"load_mode\": \"recovery_only\",");
    raw("      \"requested_capability\": ");
    json_str(RECOVERY_ARTIFACT_LOAD_CAPABILITY);
    raw_line(",");
    raw_line("      \"risk\": \"recovery_modify_ram\",");
    raw_line("      \"target\": \"recovery_lifeline\",");
    raw_line("      \"subject\": \"agent.session.serial\",");
    raw_line("      \"normal_module_load_path_used\": false,");
    raw_line("      \"normal_module_capability_used\": false,");
    raw_line("      \"separate_from\": \"cap.module.load_ephemeral\"");
    raw_line("    },");
    raw_line("    \"boundary\": {");
    raw_line("      \"schema\": \"raios.recovery_artifact_load_denial_evidence.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"status\": \"denied_missing_recovery_artifact_evidence\",");
    raw_line("      \"recovery_artifact_identity\": \"missing\",");
    raw_line("      \"recovery_artifact_trust\": \"missing\",");
    raw_line("      \"recovery_vm_test\": \"missing\",");
    raw_line("      \"recovery_local_approval\": \"missing\",");
    raw_line("      \"recovery_loader\": \"missing\",");
    raw_line("      \"recovery_rollback_evidence\": \"missing\",");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false");
    raw_line("    },");
    raw_line("    \"missing_facts\": {");
    emit_recovery_artifact_load_missing_fact(
        "recovery_artifact_identity",
        "raios.recovery_artifact_identity.v0",
        "recovery_artifact_identity_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_artifact_trust",
        "raios.recovery_artifact_trust.v0",
        "recovery_artifact_trust_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_vm_test",
        "raios.recovery_artifact_vm_test.v0",
        "recovery_vm_test_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_local_approval",
        "raios.recovery_artifact_local_approval.v0",
        "recovery_local_approval_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_loader",
        "raios.recovery_artifact_loader.v0",
        "recovery_loader_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_rollback_evidence",
        "raios.recovery_artifact_rollback_evidence.v0",
        "recovery_rollback_evidence_missing",
        false,
    );
    raw_line("    },");
    raw_line("    \"blocked_by\": [");
    raw_line("      {\"gate\": \"recovery_artifact_identity\", \"state\": \"missing\", \"reason\": \"recovery_artifact_identity_missing\"},");
    raw_line("      {\"gate\": \"recovery_artifact_trust\", \"state\": \"missing\", \"reason\": \"recovery_artifact_trust_missing\"},");
    raw_line("      {\"gate\": \"recovery_vm_test\", \"state\": \"missing\", \"reason\": \"recovery_vm_test_missing\"},");
    raw_line("      {\"gate\": \"recovery_local_approval\", \"state\": \"missing\", \"reason\": \"recovery_local_approval_missing\"},");
    raw_line("      {\"gate\": \"recovery_loader\", \"state\": \"missing\", \"reason\": \"recovery_loader_missing\"},");
    raw_line("      {\"gate\": \"recovery_rollback_evidence\", \"state\": \"missing\", \"reason\": \"recovery_rollback_evidence_missing\"}");
    raw_line("    ],");
    raw_line("    \"required\": [");
    raw_line("      \"raios.recovery_artifact_identity.v0\",");
    raw_line("      \"raios.recovery_artifact_trust.v0\",");
    raw_line("      \"raios.recovery_artifact_vm_test.v0\",");
    raw_line("      \"raios.recovery_artifact_local_approval.v0\",");
    raw_line("      \"raios.recovery_artifact_loader.v0\",");
    raw_line("      \"raios.recovery_artifact_rollback_evidence.v0\"");
    raw_line("    ],");
    raw_line("    \"evidence\": {");
    raw("      \"denial_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw_line("      \"event_scope\": \"current_boot\",");
    raw_line("      \"recovery_only_capability_id\": \"cap.recovery.load_artifact\",");
    raw_line("      \"normal_module_capability_id\": \"cap.module.load_ephemeral\",");
    raw_line("      \"normal_module_append_intent_used\": false,");
    raw_line("      \"append_payload_hash_authority\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"load_attempted\": false");
    raw_line("    }");
    raw_line("  }");
    raw_line("}");
    serial::write_raw_fmt(format_args!("RAIOS_AGENT_END {}\r\n", method));
}

pub(crate) fn emit_recovery_artifact_load_denial_event_binding(
    binding: event_log::RecoveryArtifactLoadDenialBinding,
) {
    raw(", \"bindings\": {\"schema\": \"raios.recovery_artifact_load_denial_evidence.v0\", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"status\": \"denied_missing_recovery_artifact_evidence\", \"requested_capability\": \"cap.recovery.load_artifact\", \"load_mode\": \"recovery_only\", \"separate_from\": \"cap.module.load_ephemeral\", \"normal_module_load_path_used\": false, \"normal_module_capability_used\": false, \"recovery_artifact_identity\": ");
    json_missing_state(binding.recovery_artifact_identity_missing);
    raw(", \"recovery_artifact_trust\": ");
    json_missing_state(binding.recovery_artifact_trust_missing);
    raw(", \"recovery_vm_test\": ");
    json_missing_state(binding.recovery_vm_test_missing);
    raw(", \"recovery_local_approval\": ");
    json_missing_state(binding.recovery_local_approval_missing);
    raw(", \"recovery_loader\": ");
    json_missing_state(binding.recovery_loader_missing);
    raw(", \"recovery_rollback_evidence\": ");
    json_missing_state(binding.recovery_rollback_evidence_missing);
    raw(", \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"missing_fact_schemas\": [\"raios.recovery_artifact_identity.v0\", \"raios.recovery_artifact_trust.v0\", \"raios.recovery_artifact_vm_test.v0\", \"raios.recovery_artifact_local_approval.v0\", \"raios.recovery_artifact_loader.v0\", \"raios.recovery_artifact_rollback_evidence.v0\"]}");
}

fn emit_recovery_identity_reference_object(check: &RecoveryIdentityReferenceCheck<'_>) {
    raw_line("      \"recovery_artifact_identity_reference\": {");
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
    raw_line("        \"identity_schema\": \"raios.recovery_artifact_identity.v0\",");
    raw("        \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("        \"expected_identity_reference_hash\": ");
    json_sha256_option(check.expected_identity_reference_hash);
    raw_line(",");
    raw("        \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    crlf();
    raw_line("      }");
}

fn emit_recovery_identity_retained_reference(
    check: &RecoveryIdentityReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
) {
    raw_line("      \"retained_recovery_artifact_identity_reference\": {");
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
        raw_bool(recovery_identity_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_artifact_identity.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw_line("        \"hashes\": {");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_artifact_identity.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_recovery_artifact_identity_reference_retained\",");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_recovery_trust_reference_object(check: &RecoveryTrustReferenceCheck<'_>) {
    raw_line("      \"recovery_artifact_trust_reference\": {");
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
    raw("        \"retained_recovery_artifact_identity_event_id\": ");
    json_opt_str(check.retained_identity_reference_event_id);
    raw_line(",");
    raw_line("        \"trust_schema\": \"raios.recovery_artifact_trust.v0\",");
    raw_line("        \"hashes\": {");
    raw("          \"trust_reference_hash\": ");
    json_sha256_option(check.trust_reference_hash);
    raw_line(",");
    raw("          \"expected_trust_reference_hash\": ");
    json_sha256_option(check.expected_trust_reference_hash);
    raw_line(",");
    raw("          \"identity_reference_hash\": ");
    json_sha256_option(check.identity_reference_hash);
    raw_line(",");
    raw("          \"artifact_hash\": ");
    json_sha256_option(check.artifact_hash);
    raw_line(",");
    raw("          \"trust_hash\": ");
    json_sha256_option(check.trust_hash);
    crlf();
    raw_line("        }");
    raw_line("      }");
}

fn emit_recovery_trust_retained_reference(
    check: &RecoveryTrustReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
) {
    raw_line("      \"retained_recovery_artifact_trust_reference\": {");
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
        raw_bool(recovery_trust_reference_matches(check, reference));
        raw_line(",");
        raw_line("        \"schema\": \"raios.recovery_artifact_trust.v0\",");
        raw_line("        \"status\": \"retained_hash_reference_load_still_denied\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"accepts_artifact_bytes\": false,");
        raw_line("        \"authorizes_recovery_load\": false,");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"loads_recovery_artifact\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw_line(",");
        raw("          \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw_line(",");
        raw("          \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw_line(",");
        raw("          \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"retention\": \"current_boot_ram_event_log\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"recorded_event_id\": null,");
        raw_line("        \"matches_current_reference\": false,");
        raw_line("        \"schema\": \"raios.recovery_artifact_trust.v0\",");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"no_valid_recovery_artifact_trust_reference_retained\",");
        raw_line("        \"can_move_beyond_denial\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw("      }");
}

fn emit_recovery_identity_selftest_case(case: &RecoveryIdentitySelfTestCase, comma: bool) {
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
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_recovery_trust_selftest_case(case: &RecoveryTrustSelfTestCase, comma: bool) {
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
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_recovery_artifact_load_missing_fact(
    field: &'static str,
    schema: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("      \"");
    raw(field);
    raw("\": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
    json_str(reason);
    raw(", \"authorizes_recovery_load\": false, \"loads_recovery_artifact\": false}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_load_identity_binding_fact(
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    comma: bool,
) {
    raw("      \"recovery_artifact_identity_event_id\": {\"schema\": \"raios.recovery_artifact_identity.v0\"");
    if let Some((event_id, reference)) = retained {
        raw(", \"status\": \"retained_hash_reference_only\", \"event_id\": ");
        json_event_id(event_id);
        raw(", \"retained\": true, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"retained_recovery_artifact_identity_reference_not_authorizing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"hashes\": {\"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw("}");
    } else {
        raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"recovery_artifact_identity_event_id_missing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false}");
    }
    raw("}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_load_trust_binding_fact(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    comma: bool,
) {
    raw("      \"recovery_artifact_trust_event_id\": {\"schema\": \"raios.recovery_artifact_trust.v0\"");
    if let Some((event_id, reference)) = retained {
        let mismatch = recovery_load_binding_retained_trust_mismatch(retained_identity, retained);
        raw(", \"status\": ");
        json_str(if mismatch.is_some() {
            "rejected_retained_reference"
        } else {
            "retained_hash_reference_only"
        });
        raw(", \"event_id\": ");
        json_event_id(event_id);
        raw(", \"retained\": true, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
        json_str(mismatch.unwrap_or("retained_recovery_artifact_trust_reference_not_authorizing"));
        raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_recovery_artifact_identity_event_id\": ");
        json_event_id(reference.retained_identity_reference_event_id);
        raw(", \"hashes\": {\"trust_reference_hash\": ");
        json_sha256(reference.trust_reference_hash);
        raw(", \"identity_reference_hash\": ");
        json_sha256(reference.identity_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"trust_hash\": ");
        json_sha256(reference.trust_hash);
        raw("}");
    } else {
        raw(", \"status\": \"missing\", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": \"recovery_artifact_trust_event_id_missing\", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false}");
    }
    raw("}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

fn emit_recovery_load_blocker(
    wrote: &mut bool,
    gate: &'static str,
    state: &'static str,
    reason: &'static str,
) {
    if *wrote {
        raw_line(",");
    }
    raw("        {\"gate\": ");
    json_str(gate);
    raw(", \"state\": ");
    json_str(state);
    raw(", \"reason\": ");
    json_str(reason);
    raw("}");
    *wrote = true;
}

fn emit_recovery_load_binding_check(
    check: &RecoveryLoadBindingCheck,
    _spaces: usize,
    _include_status: bool,
) {
    raw("        \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"recovery_only_capability_used\": ");
    raw_bool(check.recovery_only_capability_used);
    raw_line(",");
    raw("        \"accepts_normal_module_authority\": ");
    raw_bool(check.accepts_normal_module_authority);
    raw_line(",");
    raw("        \"append_payload_hash_authority\": ");
    raw_bool(check.append_payload_hash_authority);
    raw_line(",");
    raw("        \"can_move_beyond_denial\": ");
    raw_bool(check.can_move_beyond_denial);
    raw_line(",");
    raw("        \"loads_recovery_artifact\": ");
    raw_bool(check.loads_recovery_artifact);
    raw_line(",");
    raw("        \"loads_normal_module\": ");
    raw_bool(check.loads_normal_module);
    raw_line(",");
    raw("        \"creates_durable_records\": ");
    raw_bool(check.creates_durable_records);
    raw_line(",");
    raw("        \"installs_rollback_plan\": ");
    raw_bool(check.installs_rollback_plan);
    raw_line(",");
    raw("        \"service_inventory_change\": ");
    json_str(check.service_inventory_change);
    raw_line(",");
    raw("        \"load_attempted\": ");
    raw_bool(check.load_attempted);
    crlf();
}

fn emit_recovery_load_binding_selftest_case(case: &RecoveryLoadBindingSelfTestCase, comma: bool) {
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
    raw(", \"can_move_beyond_denial\": false, \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"load_attempted\": false, \"normal_module_capability_accepted\": false, \"append_payload_hash_authority\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn recovery_load_binding_selftest_cases(
) -> [RecoveryLoadBindingSelfTestCase; RECOVERY_LOAD_BINDING_SELFTEST_CASES] {
    let valid = recovery_load_binding_available_candidate();

    let mut identity_missing = valid;
    identity_missing.identity = recovery_evidence_missing();
    let mut identity_previous_boot = valid;
    identity_previous_boot.identity.current_boot = false;
    let mut identity_wrong_schema = valid;
    identity_wrong_schema.identity.schema_ok = false;

    let mut trust_missing = valid;
    trust_missing.trust = recovery_evidence_missing();
    let mut vm_test_missing = valid;
    vm_test_missing.vm_test = recovery_evidence_missing();
    let mut local_approval_missing = valid;
    local_approval_missing.local_approval = recovery_evidence_missing();
    let mut loader_missing = valid;
    loader_missing.loader = recovery_evidence_missing();
    let mut rollback_missing = valid;
    rollback_missing.rollback_evidence = recovery_evidence_missing();

    let mut module_capability = valid;
    module_capability.requested_capability = "cap.module.load_ephemeral";
    module_capability.normal_module_capability_substituted = true;
    let mut module_append_intent = valid;
    module_append_intent.normal_module_append_intent_substituted = true;
    let mut append_payload_hash = valid;
    append_payload_hash.append_payload_hash_claimed_authority = true;
    let mut module_writer_facts = valid;
    module_writer_facts.normal_module_writer_facts_substituted = true;
    let mut module_service_slot = valid;
    module_service_slot.normal_module_service_slot_substituted = true;

    [
        recovery_load_binding_selftest_case(
            "missing_recovery_artifact_identity_event_id",
            "missing",
            "recovery_artifact_identity_event_id_missing",
            identity_missing,
        ),
        recovery_load_binding_selftest_case(
            "previous_boot_recovery_artifact_identity_event_id",
            "rejected",
            "recovery_artifact_identity_event_id_not_current_boot",
            identity_previous_boot,
        ),
        recovery_load_binding_selftest_case(
            "wrong_schema_recovery_artifact_identity_event_id",
            "rejected",
            "recovery_artifact_identity_schema_mismatch",
            identity_wrong_schema,
        ),
        recovery_load_binding_selftest_case(
            "missing_recovery_artifact_trust_event_id",
            "missing",
            "recovery_artifact_trust_event_id_missing",
            trust_missing,
        ),
        recovery_load_binding_selftest_case(
            "missing_recovery_vm_test_event_id",
            "missing",
            "recovery_vm_test_event_id_missing",
            vm_test_missing,
        ),
        recovery_load_binding_selftest_case(
            "missing_recovery_local_approval_event_id",
            "missing",
            "recovery_local_approval_event_id_missing",
            local_approval_missing,
        ),
        recovery_load_binding_selftest_case(
            "missing_recovery_loader_event_id",
            "missing",
            "recovery_loader_event_id_missing",
            loader_missing,
        ),
        recovery_load_binding_selftest_case(
            "missing_recovery_rollback_evidence_event_id",
            "missing",
            "recovery_rollback_evidence_event_id_missing",
            rollback_missing,
        ),
        recovery_load_binding_selftest_case(
            "module_load_ephemeral_capability_substituted",
            "rejected",
            "recovery_load_requires_cap_recovery_load_artifact",
            module_capability,
        ),
        recovery_load_binding_selftest_case(
            "normal_module_append_intent_substituted",
            "rejected",
            "normal_module_append_intent_not_recovery_authority",
            module_append_intent,
        ),
        recovery_load_binding_selftest_case(
            "append_payload_hash_claimed_as_authority",
            "rejected",
            "append_payload_hash_not_recovery_authority",
            append_payload_hash,
        ),
        recovery_load_binding_selftest_case(
            "normal_module_writer_facts_substituted",
            "rejected",
            "normal_module_writer_facts_not_recovery_authority",
            module_writer_facts,
        ),
        recovery_load_binding_selftest_case(
            "normal_module_service_slot_substituted",
            "rejected",
            "normal_module_service_slot_not_recovery_authority",
            module_service_slot,
        ),
        recovery_load_binding_selftest_case(
            "available_recovery_binding_still_denied",
            "available_non_authorizing",
            "recovery_lifeline_protocol_missing",
            valid,
        ),
    ]
}

fn recovery_load_binding_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: RecoveryLoadBindingCandidate,
) -> RecoveryLoadBindingSelfTestCase {
    let actual = evaluate_recovery_load_binding(candidate);
    RecoveryLoadBindingSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.can_move_beyond_denial
            && !actual.accepts_normal_module_authority
            && !actual.append_payload_hash_authority
            && !actual.loads_recovery_artifact
            && !actual.loads_normal_module
            && !actual.creates_durable_records
            && !actual.installs_rollback_plan
            && method_eq(actual.service_inventory_change, "none")
            && !actual.load_attempted,
    }
}

fn evaluate_recovery_load_binding(
    candidate: RecoveryLoadBindingCandidate,
) -> RecoveryLoadBindingCheck {
    let (status, reason) = if !method_eq(
        candidate.requested_capability,
        RECOVERY_ARTIFACT_LOAD_CAPABILITY,
    ) || candidate.normal_module_capability_substituted
    {
        (
            "rejected",
            "recovery_load_requires_cap_recovery_load_artifact",
        )
    } else if candidate.normal_module_append_intent_substituted {
        (
            "rejected",
            "normal_module_append_intent_not_recovery_authority",
        )
    } else if candidate.append_payload_hash_claimed_authority {
        ("rejected", "append_payload_hash_not_recovery_authority")
    } else if candidate.normal_module_writer_facts_substituted {
        (
            "rejected",
            "normal_module_writer_facts_not_recovery_authority",
        )
    } else if candidate.normal_module_service_slot_substituted {
        (
            "rejected",
            "normal_module_service_slot_not_recovery_authority",
        )
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.identity,
        "missing",
        "recovery_artifact_identity_event_id_missing",
        "rejected",
        "recovery_artifact_identity_event_id_not_current_boot",
        "recovery_artifact_identity_schema_mismatch",
    ) {
        result
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.trust,
        "missing",
        "recovery_artifact_trust_event_id_missing",
        "rejected",
        "recovery_artifact_trust_event_id_not_current_boot",
        "recovery_artifact_trust_schema_mismatch",
    ) {
        result
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.vm_test,
        "missing",
        "recovery_vm_test_event_id_missing",
        "rejected",
        "recovery_vm_test_event_id_not_current_boot",
        "recovery_vm_test_schema_mismatch",
    ) {
        result
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.local_approval,
        "missing",
        "recovery_local_approval_event_id_missing",
        "rejected",
        "recovery_local_approval_event_id_not_current_boot",
        "recovery_local_approval_schema_mismatch",
    ) {
        result
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.loader,
        "missing",
        "recovery_loader_event_id_missing",
        "rejected",
        "recovery_loader_event_id_not_current_boot",
        "recovery_loader_schema_mismatch",
    ) {
        result
    } else if let Some(result) = evaluate_recovery_evidence(
        candidate.rollback_evidence,
        "missing",
        "recovery_rollback_evidence_event_id_missing",
        "rejected",
        "recovery_rollback_evidence_event_id_not_current_boot",
        "recovery_rollback_evidence_schema_mismatch",
    ) {
        result
    } else {
        (
            "available_non_authorizing",
            "recovery_lifeline_protocol_missing",
        )
    };

    RecoveryLoadBindingCheck {
        status,
        reason,
        recovery_only_capability_used: method_eq(
            candidate.requested_capability,
            RECOVERY_ARTIFACT_LOAD_CAPABILITY,
        ) && !candidate.normal_module_capability_substituted,
        accepts_normal_module_authority: false,
        append_payload_hash_authority: false,
        can_move_beyond_denial: false,
        loads_recovery_artifact: false,
        loads_normal_module: false,
        creates_durable_records: false,
        installs_rollback_plan: false,
        service_inventory_change: "none",
        load_attempted: false,
    }
}

fn evaluate_recovery_evidence(
    evidence: RecoveryEvidenceCandidate,
    missing_status: &'static str,
    missing_reason: &'static str,
    rejected_status: &'static str,
    stale_reason: &'static str,
    schema_reason: &'static str,
) -> Option<(&'static str, &'static str)> {
    if !evidence.retained {
        Some((missing_status, missing_reason))
    } else if !evidence.current_boot {
        Some((rejected_status, stale_reason))
    } else if !evidence.schema_ok {
        Some((rejected_status, schema_reason))
    } else if !evidence.binding_ok {
        Some((rejected_status, evidence.binding_reason))
    } else {
        None
    }
}

fn recovery_load_binding_candidate_from_retained(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
) -> RecoveryLoadBindingCandidate {
    let mut candidate = recovery_load_binding_missing_candidate();
    if retained_identity.is_some() {
        candidate.identity = recovery_evidence_available();
    }
    if retained_trust.is_some() {
        candidate.trust = if let Some(reason) =
            recovery_load_binding_retained_trust_mismatch(retained_identity, retained_trust)
        {
            recovery_evidence_rejected(reason)
        } else {
            recovery_evidence_available()
        };
    }
    candidate
}

fn recovery_load_binding_retained_trust_mismatch(
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
) -> Option<&'static str> {
    let Some((identity_event_id, identity_reference)) = retained_identity else {
        return None;
    };
    let Some((_trust_event_id, trust_reference)) = retained_trust else {
        return None;
    };
    if trust_reference.retained_identity_reference_event_id != identity_event_id {
        return Some("recovery_artifact_trust_identity_event_id_mismatch");
    }
    if trust_reference.identity_reference_hash != identity_reference.identity_reference_hash {
        return Some("recovery_artifact_trust_identity_reference_hash_mismatch");
    }
    if trust_reference.artifact_hash != identity_reference.artifact_hash {
        return Some("recovery_artifact_trust_artifact_hash_mismatch");
    }
    None
}

fn recovery_load_binding_missing_candidate() -> RecoveryLoadBindingCandidate {
    RecoveryLoadBindingCandidate {
        requested_capability: RECOVERY_ARTIFACT_LOAD_CAPABILITY,
        identity: recovery_evidence_missing(),
        trust: recovery_evidence_missing(),
        vm_test: recovery_evidence_missing(),
        local_approval: recovery_evidence_missing(),
        loader: recovery_evidence_missing(),
        rollback_evidence: recovery_evidence_missing(),
        normal_module_capability_substituted: false,
        normal_module_append_intent_substituted: false,
        append_payload_hash_claimed_authority: false,
        normal_module_writer_facts_substituted: false,
        normal_module_service_slot_substituted: false,
    }
}

fn recovery_load_binding_available_candidate() -> RecoveryLoadBindingCandidate {
    RecoveryLoadBindingCandidate {
        requested_capability: RECOVERY_ARTIFACT_LOAD_CAPABILITY,
        identity: recovery_evidence_available(),
        trust: recovery_evidence_available(),
        vm_test: recovery_evidence_available(),
        local_approval: recovery_evidence_available(),
        loader: recovery_evidence_available(),
        rollback_evidence: recovery_evidence_available(),
        normal_module_capability_substituted: false,
        normal_module_append_intent_substituted: false,
        append_payload_hash_claimed_authority: false,
        normal_module_writer_facts_substituted: false,
        normal_module_service_slot_substituted: false,
    }
}

fn recovery_evidence_available() -> RecoveryEvidenceCandidate {
    RecoveryEvidenceCandidate {
        retained: true,
        current_boot: true,
        schema_ok: true,
        binding_ok: true,
        binding_reason: "",
    }
}

fn recovery_evidence_missing() -> RecoveryEvidenceCandidate {
    RecoveryEvidenceCandidate {
        retained: false,
        current_boot: true,
        schema_ok: true,
        binding_ok: true,
        binding_reason: "",
    }
}

fn recovery_evidence_rejected(reason: &'static str) -> RecoveryEvidenceCandidate {
    RecoveryEvidenceCandidate {
        retained: true,
        current_boot: true,
        schema_ok: true,
        binding_ok: false,
        binding_reason: reason,
    }
}

fn parse_recovery_identity_reference(arg: &str) -> RecoveryIdentityReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let identity_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryIdentityReferenceCheck {
        has_reference: identity_reference_hash.is_some(),
        arity_valid: identity_reference_hash.is_some()
            && artifact_hash.is_some()
            && extra.is_none(),
        scope,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        expected_identity_reference_hash: None,
        status: "missing",
        reason: "recovery_artifact_identity_reference_absent",
        valid: false,
    };
    evaluate_recovery_identity_reference(input)
}

fn evaluate_recovery_identity_reference(
    input: RecoveryIdentityReferenceCheck<'_>,
) -> RecoveryIdentityReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_identity_reference_check(
            input,
            None,
            "missing",
            "recovery_artifact_identity_reference_absent",
            false,
        );
    }
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_identity_reference_check(
            input,
            None,
            if input.has_reference {
                "invalid_reference"
            } else {
                "missing"
            },
            if input.has_reference {
                "recovery_artifact_identity_reference_invalid_hash"
            } else {
                "recovery_artifact_identity_reference_absent"
            },
            false,
        );
    };
    if !input.arity_valid {
        return recovery_identity_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_identity_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_identity_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_artifact_identity_reference_scope_must_be_current_boot",
            false,
        );
    }
    let expected =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    if input.identity_reference_hash != Some(expected) {
        return recovery_identity_reference_check(
            input,
            Some(expected),
            "mismatched_identity_reference_hash",
            "recovery_artifact_identity_reference_hash_mismatch",
            false,
        );
    }
    recovery_identity_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_artifact_identity_reference_valid_but_trust_and_loader_missing",
        true,
    )
}

fn recovery_identity_reference_check<'a>(
    input: RecoveryIdentityReferenceCheck<'a>,
    expected_identity_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryIdentityReferenceCheck<'a> {
    RecoveryIdentityReferenceCheck {
        expected_identity_reference_hash,
        status,
        reason,
        valid,
        ..input
    }
}

fn parse_recovery_trust_reference(
    arg: &str,
    require_live_retained: bool,
) -> RecoveryTrustReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let trust_reference_hash = parts.next();
    let retained_identity_reference_event_id = parts.next();
    let identity_reference_hash = parts.next();
    let artifact_hash = parts.next();
    let trust_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryTrustReferenceInput {
        has_reference: trust_reference_hash.is_some(),
        arity_valid: trust_reference_hash.is_some()
            && retained_identity_reference_event_id.is_some()
            && identity_reference_hash.is_some()
            && artifact_hash.is_some()
            && trust_hash.is_some()
            && extra.is_none(),
        scope,
        trust_reference_hash: trust_reference_hash.and_then(parse_sha256_ref),
        retained_identity_reference_event_id,
        identity_reference_hash: identity_reference_hash.and_then(parse_sha256_ref),
        artifact_hash: artifact_hash.and_then(parse_sha256_ref),
        trust_hash: trust_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_trust_reference(input, require_live_retained)
}

fn evaluate_recovery_trust_reference(
    input: RecoveryTrustReferenceInput<'_>,
    require_live_retained: bool,
) -> RecoveryTrustReferenceCheck<'_> {
    if !input.has_reference {
        return recovery_trust_reference_check(
            input,
            None,
            "missing",
            "recovery_artifact_trust_reference_absent",
            false,
        );
    }
    let Some(identity_event_id) = input.retained_identity_reference_event_id else {
        return recovery_trust_reference_check(
            input,
            None,
            if input.has_reference {
                "invalid_reference"
            } else {
                "missing"
            },
            if input.has_reference {
                "recovery_artifact_trust_reference_invalid_hash"
            } else {
                "recovery_artifact_trust_reference_absent"
            },
            false,
        );
    };
    let Some(identity_reference_hash) = input.identity_reference_hash else {
        return recovery_trust_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_trust_reference_invalid_hash",
            false,
        );
    };
    let Some(artifact_hash) = input.artifact_hash else {
        return recovery_trust_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_trust_reference_invalid_hash",
            false,
        );
    };
    let Some(trust_hash) = input.trust_hash else {
        return recovery_trust_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_trust_reference_invalid_hash",
            false,
        );
    };
    if !input.arity_valid {
        return recovery_trust_reference_check(
            input,
            None,
            "invalid_reference",
            "recovery_artifact_trust_reference_arity_invalid",
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_trust_reference_check(
            input,
            None,
            "stale_or_non_current_boot_reference",
            "recovery_artifact_trust_reference_scope_must_be_current_boot",
            false,
        );
    }
    if !current_boot_event_id_str(identity_event_id) {
        return recovery_trust_reference_check(
            input,
            None,
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            false,
        );
    }
    let expected = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    if input.trust_reference_hash != Some(expected) {
        return recovery_trust_reference_check(
            input,
            Some(expected),
            "mismatched_trust_reference_hash",
            "recovery_artifact_trust_reference_hash_mismatch",
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_trust_live_identity_mismatch(&input) {
            return recovery_trust_reference_check(
                input,
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_trust_reference_check(
        input,
        Some(expected),
        "valid_hash_reference_load_still_denied",
        "recovery_artifact_trust_reference_valid_but_vm_test_and_loader_missing",
        true,
    )
}

fn recovery_trust_reference_check<'a>(
    input: RecoveryTrustReferenceInput<'a>,
    expected_trust_reference_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryTrustReferenceCheck<'a> {
    RecoveryTrustReferenceCheck {
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        trust_reference_hash: input.trust_reference_hash,
        retained_identity_reference_event_id: input.retained_identity_reference_event_id,
        identity_reference_hash: input.identity_reference_hash,
        artifact_hash: input.artifact_hash,
        trust_hash: input.trust_hash,
        expected_trust_reference_hash,
        status,
        reason,
        valid,
    }
}

fn recovery_trust_live_identity_mismatch(
    input: &RecoveryTrustReferenceInput<'_>,
) -> Option<&'static str> {
    let retained_identity_reference_event_id =
        parse_current_boot_event_id(input.retained_identity_reference_event_id?)?;
    let Some((latest_event_id, identity_reference)) =
        event_log::latest_recovery_artifact_identity_reference()
    else {
        return Some("recovery_artifact_identity_reference_missing");
    };
    if latest_event_id != retained_identity_reference_event_id {
        return Some("recovery_artifact_identity_reference_event_id_mismatch");
    }
    if Some(identity_reference.identity_reference_hash) != input.identity_reference_hash {
        return Some("recovery_artifact_identity_reference_hash_mismatch");
    }
    if Some(identity_reference.artifact_hash) != input.artifact_hash {
        return Some("recovery_artifact_identity_artifact_hash_mismatch");
    }
    None
}

fn recovery_identity_selftest_cases(
) -> [RecoveryIdentitySelfTestCase; RECOVERY_IDENTITY_SELFTEST_CASES] {
    let artifact_hash = [0x91; 32];
    let valid_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let valid = RecoveryIdentityReferenceCheck {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        identity_reference_hash: Some(valid_hash),
        artifact_hash: Some(artifact_hash),
        expected_identity_reference_hash: None,
        status: "missing",
        reason: "missing",
        valid: false,
    };
    [
        recovery_identity_selftest_case(
            "absent_reference",
            "missing",
            "recovery_artifact_identity_reference_absent",
            evaluate_recovery_identity_reference(RecoveryIdentityReferenceCheck {
                has_reference: false,
                ..valid
            }),
        ),
        recovery_identity_selftest_case(
            "accepted_current_boot_identity_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_artifact_identity_reference_valid_but_trust_and_loader_missing",
            evaluate_recovery_identity_reference(valid),
        ),
        recovery_identity_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_artifact_identity_reference_scope_must_be_current_boot",
            evaluate_recovery_identity_reference(RecoveryIdentityReferenceCheck {
                scope: "previous_boot",
                ..valid
            }),
        ),
        recovery_identity_selftest_case(
            "wrong_schema_identity_reference",
            "rejected",
            "recovery_artifact_identity_reference_wrong_schema_or_variant",
            RecoveryIdentityReferenceCheck {
                status: "rejected",
                reason: "recovery_artifact_identity_reference_wrong_schema_or_variant",
                valid: false,
                ..valid
            },
        ),
        recovery_identity_selftest_case(
            "substituted_identity_reference_record",
            "rejected",
            "recovery_artifact_identity_reference_substituted_record",
            RecoveryIdentityReferenceCheck {
                status: "rejected",
                reason: "recovery_artifact_identity_reference_substituted_record",
                valid: false,
                ..valid
            },
        ),
        recovery_identity_selftest_case(
            "identity_reference_hash_mismatch",
            "mismatched_identity_reference_hash",
            "recovery_artifact_identity_reference_hash_mismatch",
            evaluate_recovery_identity_reference(RecoveryIdentityReferenceCheck {
                identity_reference_hash: Some([0x92; 32]),
                ..valid
            }),
        ),
    ]
}

fn recovery_identity_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryIdentityReferenceCheck<'_>,
) -> RecoveryIdentitySelfTestCase {
    RecoveryIdentitySelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

fn recovery_trust_selftest_cases() -> [RecoveryTrustSelfTestCase; RECOVERY_TRUST_SELFTEST_CASES] {
    let artifact_hash = [0x91; 32];
    let trust_hash = [0x93; 32];
    let identity_reference_hash =
        module_evidence::computed_recovery_artifact_identity_reference_hash(artifact_hash);
    let identity_event_id = "event.current_boot.00000031";
    let valid_hash = module_evidence::computed_recovery_artifact_trust_reference_hash(
        module_evidence::RecoveryArtifactTrustReferenceHashInput {
            retained_identity_reference_event_id: identity_event_id,
            identity_reference_hash,
            artifact_hash,
            trust_hash,
        },
    );
    let valid = RecoveryTrustReferenceInput {
        has_reference: true,
        arity_valid: true,
        scope: "current_boot",
        trust_reference_hash: Some(valid_hash),
        retained_identity_reference_event_id: Some(identity_event_id),
        identity_reference_hash: Some(identity_reference_hash),
        artifact_hash: Some(artifact_hash),
        trust_hash: Some(trust_hash),
    };
    [
        recovery_trust_selftest_case(
            "absent_reference",
            "missing",
            "recovery_artifact_trust_reference_absent",
            evaluate_recovery_trust_reference(
                RecoveryTrustReferenceInput {
                    has_reference: false,
                    ..valid
                },
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "accepted_current_boot_trust_still_denied",
            "valid_hash_reference_load_still_denied",
            "recovery_artifact_trust_reference_valid_but_vm_test_and_loader_missing",
            evaluate_recovery_trust_reference(valid, false),
        ),
        recovery_trust_selftest_case(
            "stale_previous_boot_reference",
            "stale_or_non_current_boot_reference",
            "recovery_artifact_trust_reference_scope_must_be_current_boot",
            evaluate_recovery_trust_reference(
                RecoveryTrustReferenceInput {
                    scope: "previous_boot",
                    ..valid
                },
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "retained_identity_event_not_current_boot",
            "rejected",
            "retained_recovery_artifact_identity_event_id_not_current_boot",
            evaluate_recovery_trust_reference(
                RecoveryTrustReferenceInput {
                    retained_identity_reference_event_id: Some("event.previous_boot.00000031"),
                    ..valid
                },
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "retained_identity_missing",
            "rejected",
            "recovery_artifact_identity_reference_missing",
            recovery_trust_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_identity_reference_missing",
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "retained_identity_wrong_schema_or_variant",
            "rejected",
            "recovery_artifact_identity_reference_wrong_schema_or_variant",
            recovery_trust_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_identity_reference_wrong_schema_or_variant",
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "substituted_identity_reference_record",
            "rejected",
            "recovery_artifact_identity_reference_substituted_record",
            recovery_trust_reference_check(
                valid,
                Some(valid_hash),
                "rejected",
                "recovery_artifact_identity_reference_substituted_record",
                false,
            ),
        ),
        recovery_trust_selftest_case(
            "trust_reference_hash_mismatch",
            "mismatched_trust_reference_hash",
            "recovery_artifact_trust_reference_hash_mismatch",
            evaluate_recovery_trust_reference(
                RecoveryTrustReferenceInput {
                    trust_reference_hash: Some([0x94; 32]),
                    ..valid
                },
                false,
            ),
        ),
    ]
}

fn recovery_trust_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: RecoveryTrustReferenceCheck<'_>,
) -> RecoveryTrustSelfTestCase {
    RecoveryTrustSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: method_eq(check.status, expected_status)
            && method_eq(check.reason, expected_reason)
            && check.valid == method_eq(expected_status, "valid_hash_reference_load_still_denied"),
    }
}

fn recovery_identity_binding_from_check(
    check: &RecoveryIdentityReferenceCheck<'_>,
) -> Option<event_log::RecoveryArtifactIdentityReference> {
    Some(event_log::RecoveryArtifactIdentityReference {
        identity_reference_hash: check.identity_reference_hash?,
        artifact_hash: check.artifact_hash?,
    })
}

fn recovery_trust_binding_from_check(
    check: &RecoveryTrustReferenceCheck<'_>,
) -> Option<event_log::RecoveryArtifactTrustReference> {
    Some(event_log::RecoveryArtifactTrustReference {
        trust_reference_hash: check.trust_reference_hash?,
        retained_identity_reference_event_id: parse_current_boot_event_id(
            check.retained_identity_reference_event_id?,
        )?,
        identity_reference_hash: check.identity_reference_hash?,
        artifact_hash: check.artifact_hash?,
        trust_hash: check.trust_hash?,
    })
}

fn recovery_identity_reference_matches(
    check: &RecoveryIdentityReferenceCheck<'_>,
    reference: event_log::RecoveryArtifactIdentityReference,
) -> bool {
    check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
}

fn recovery_trust_reference_matches(
    check: &RecoveryTrustReferenceCheck<'_>,
    reference: event_log::RecoveryArtifactTrustReference,
) -> bool {
    check.trust_reference_hash == Some(reference.trust_reference_hash)
        && check
            .retained_identity_reference_event_id
            .and_then(parse_current_boot_event_id)
            == Some(reference.retained_identity_reference_event_id)
        && check.identity_reference_hash == Some(reference.identity_reference_hash)
        && check.artifact_hash == Some(reference.artifact_hash)
        && check.trust_hash == Some(reference.trust_hash)
}

fn recovery_identity_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "recovery.identity_diagnostic") {
        "recovery.identity_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

fn recovery_trust_diagnostic_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "recovery.trust_diagnostic") {
        "recovery.trust_diagnostic".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}

fn json_missing_state(missing: bool) {
    json_str(if missing { "missing" } else { "available" });
}
