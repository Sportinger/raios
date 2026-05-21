use crate::{
    agent_protocol_support::{
        begin_response, crlf, end_response, json_event_id, json_str, method_eq, method_head_eq,
        raw, raw_bool, raw_fmt, raw_line,
    },
    event_log, serial,
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

#[derive(Clone, Copy)]
struct RecoveryEvidenceCandidate {
    retained: bool,
    current_boot: bool,
    schema_ok: bool,
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

pub(crate) fn emit_recovery_artifact_load_binding() {
    let live = evaluate_recovery_load_binding(recovery_load_binding_missing_candidate());

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
    emit_recovery_artifact_load_missing_fact(
        "recovery_artifact_identity_event_id",
        "raios.recovery_artifact_identity.v0",
        "recovery_artifact_identity_event_id_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_artifact_trust_event_id",
        "raios.recovery_artifact_trust.v0",
        "recovery_artifact_trust_event_id_missing",
        true,
    );
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
    raw_line("        {\"gate\": \"recovery_artifact_identity_event_id\", \"state\": \"missing\", \"reason\": \"recovery_artifact_identity_event_id_missing\"},");
    raw_line("        {\"gate\": \"recovery_artifact_trust_event_id\", \"state\": \"missing\", \"reason\": \"recovery_artifact_trust_event_id_missing\"},");
    raw_line("        {\"gate\": \"recovery_vm_test_event_id\", \"state\": \"missing\", \"reason\": \"recovery_vm_test_event_id_missing\"},");
    raw_line("        {\"gate\": \"recovery_local_approval_event_id\", \"state\": \"missing\", \"reason\": \"recovery_local_approval_event_id_missing\"},");
    raw_line("        {\"gate\": \"recovery_loader_event_id\", \"state\": \"missing\", \"reason\": \"recovery_loader_event_id_missing\"},");
    raw_line("        {\"gate\": \"recovery_rollback_evidence_event_id\", \"state\": \"missing\", \"reason\": \"recovery_rollback_evidence_event_id_missing\"}");
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
    } else {
        None
    }
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
    }
}

fn recovery_evidence_missing() -> RecoveryEvidenceCandidate {
    RecoveryEvidenceCandidate {
        retained: false,
        current_boot: true,
        schema_ok: true,
    }
}

fn json_missing_state(missing: bool) {
    json_str(if missing { "missing" } else { "available" });
}
