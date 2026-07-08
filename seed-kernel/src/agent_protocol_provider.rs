use alloc::{vec, vec::Vec};
use sha2::{Digest, Sha256};

use crate::{
    agent_protocol_support::{
        begin_response, crlf, emit_export_gate, emit_record_fields, end_response, indent,
        json_current_boot_id, json_event_id, json_event_id_option, json_opt_str, json_sha256,
        json_sha256_option, json_str, method_eq, method_head_eq, raw, raw_bool, raw_fmt, raw_line,
        record_bool as b, record_field as f, record_str as s,
    },
    agent_protocol_system::{
        emit_capability_ids, emit_problem_objects, emit_service_ids, emit_status_state_at,
        CAPABILITIES,
    },
    event_log, memory_store, provider, serial, service_inventory, system_status,
    system_status::{RowState, SystemSnapshot},
    ui, wifi,
};
use raios_core::record::{sha256_of_json, Value as V};
#[derive(Clone, Copy)]
struct ProjectionFieldSpec {
    field: &'static str,
    classification: &'static str,
    action: &'static str,
    reason: &'static str,
}

pub(crate) struct ProviderContextEvidence {
    pub projected_packet_hash: [u8; 32],
    pub exported_field_list_hash: [u8; 32],
    pub omitted_field_list_hash: [u8; 32],
    pub redaction_policy_hash: [u8; 32],
    pub field_classification_hash: [u8; 32],
    pub token_budget_hash: [u8; 32],
}

impl ProviderContextEvidence {
    pub(crate) fn event_hashes(&self) -> event_log::ProviderContextHashes {
        event_log::ProviderContextHashes {
            projected_packet_hash: self.projected_packet_hash,
            exported_field_list_hash: self.exported_field_list_hash,
            omitted_field_list_hash: self.omitted_field_list_hash,
            redaction_policy_hash: self.redaction_policy_hash,
            field_classification_hash: self.field_classification_hash,
            token_budget_hash: self.token_budget_hash,
        }
    }
}

pub(crate) fn provider_minimal_context_evidence_for_runtime(
    runtime: ui::RuntimeStatus,
) -> ProviderContextEvidence {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    provider_context_evidence(&status, &provider)
}

const PROVIDER_MINIMAL_INCLUDED_FIELDS: &[ProjectionFieldSpec] = &[
    ProjectionFieldSpec {
        field: "schema",
        classification: "public",
        action: "include",
        reason: "context schema id is non-secret and required for decoding",
    },
    ProjectionFieldSpec {
        field: "purpose",
        classification: "public",
        action: "include",
        reason: "bounded task purpose, not raw user prompt",
    },
    ProjectionFieldSpec {
        field: "profile",
        classification: "public",
        action: "include",
        reason: "redaction profile id",
    },
    ProjectionFieldSpec {
        field: "scope",
        classification: "public",
        action: "include",
        reason: "current_boot scope marker",
    },
    ProjectionFieldSpec {
        field: "budget.target_tokens",
        classification: "public",
        action: "include",
        reason: "token budget target for the projection",
    },
    ProjectionFieldSpec {
        field: "budget.estimated_tokens",
        classification: "public",
        action: "include",
        reason: "bounded estimate only",
    },
    ProjectionFieldSpec {
        field: "authority_order[]",
        classification: "public",
        action: "include",
        reason: "authority labels from ADR 0004",
    },
    ProjectionFieldSpec {
        field: "included.*[]",
        classification: "public",
        action: "include",
        reason: "stable record ids only",
    },
    ProjectionFieldSpec {
        field: "current.os.*",
        classification: "public",
        action: "include",
        reason: "product identity and stage",
    },
    ProjectionFieldSpec {
        field: "current.status.*",
        classification: "public",
        action: "include",
        reason: "coarse subsystem states only",
    },
    ProjectionFieldSpec {
        field: "current.provider.selected",
        classification: "public",
        action: "include",
        reason: "provider family only",
    },
    ProjectionFieldSpec {
        field: "current.provider.route",
        classification: "public",
        action: "include",
        reason: "canonical route name without credentials",
    },
    ProjectionFieldSpec {
        field: "current.provider.api_key_state",
        classification: "public",
        action: "include",
        reason: "state marker only; never the key",
    },
    ProjectionFieldSpec {
        field: "current.provider.direct_phase",
        classification: "public",
        action: "include",
        reason: "coarse provider phase",
    },
    ProjectionFieldSpec {
        field: "current.provider.direct_endpoint",
        classification: "public",
        action: "include",
        reason: "canonical provider endpoint",
    },
    ProjectionFieldSpec {
        field: "current.provider.direct_model",
        classification: "public",
        action: "include",
        reason: "model id",
    },
    ProjectionFieldSpec {
        field: "current.provider.trust_state",
        classification: "public",
        action: "include",
        reason: "required for fail-closed provider policy",
    },
    ProjectionFieldSpec {
        field: "current.provider.pin_kind",
        classification: "public",
        action: "include",
        reason: "pin type only",
    },
    ProjectionFieldSpec {
        field: "current.provider.pin_id",
        classification: "public",
        action: "include",
        reason: "short non-secret pin identifier only",
    },
    ProjectionFieldSpec {
        field: "current.provider.pin_slot",
        classification: "public",
        action: "include",
        reason: "active or rotation slot marker only",
    },
    ProjectionFieldSpec {
        field: "current.provider.pin_rotation_policy",
        classification: "public",
        action: "include",
        reason: "explicit pin-only rotation posture",
    },
    ProjectionFieldSpec {
        field: "current.provider.pin_rotation_id",
        classification: "public",
        action: "include",
        reason: "short non-secret standby pin identifier only",
    },
    ProjectionFieldSpec {
        field: "current.provider.development_bypass",
        classification: "public",
        action: "include",
        reason: "must be visible because it blocks trusted export",
    },
    ProjectionFieldSpec {
        field: "current.provider.verifier_decision.*",
        classification: "public",
        action: "include",
        reason: "typed trust verifier outcome without certificate bytes or secrets",
    },
    ProjectionFieldSpec {
        field: "current.services[]",
        classification: "public",
        action: "include",
        reason: "stable service ids only",
    },
    ProjectionFieldSpec {
        field: "current.capabilities[]",
        classification: "public",
        action: "include",
        reason: "capability ids and denied mutation posture",
    },
    ProjectionFieldSpec {
        field: "current.problems[].id",
        classification: "public",
        action: "include",
        reason: "stable problem ids",
    },
    ProjectionFieldSpec {
        field: "current.problems[].severity",
        classification: "public",
        action: "include",
        reason: "coarse severity",
    },
    ProjectionFieldSpec {
        field: "current.problems[].summary",
        classification: "public",
        action: "include",
        reason: "stable scrubbed summaries only",
    },
    ProjectionFieldSpec {
        field: "records[].id",
        classification: "public",
        action: "include",
        reason: "stable locators",
    },
    ProjectionFieldSpec {
        field: "records[].kind",
        classification: "public",
        action: "include",
        reason: "record kind labels",
    },
    ProjectionFieldSpec {
        field: "records[].authority",
        classification: "public",
        action: "include",
        reason: "authority labels",
    },
    ProjectionFieldSpec {
        field: "records[].classification",
        classification: "public",
        action: "include",
        reason: "classification labels",
    },
    ProjectionFieldSpec {
        field: "records[].summary",
        classification: "public",
        action: "include",
        reason: "only summaries for records classified public",
    },
];

const PROVIDER_MINIMAL_OMITTED_FIELDS: &[ProjectionFieldSpec] = &[
    ProjectionFieldSpec {
        field: "source_schemas",
        classification: "local_only",
        action: "omit",
        reason: "local provenance list; the provider packet names only its projection schema",
    },
    ProjectionFieldSpec {
        field: "system.snapshot.raw",
        classification: "local_only",
        action: "omit",
        reason: "raw system.snapshot is never attached to provider context",
    },
    ProjectionFieldSpec {
        field: "details.*.detail",
        classification: "local_only",
        action: "omit",
        reason: "detail strings may contain IPs, PCI data, topology, request ids, or hashes",
    },
    ProjectionFieldSpec {
        field: "network.ip",
        classification: "local_only",
        action: "omit",
        reason: "local network address",
    },
    ProjectionFieldSpec {
        field: "network.gateway",
        classification: "local_only",
        action: "omit",
        reason: "local network topology",
    },
    ProjectionFieldSpec {
        field: "network.dns",
        classification: "local_only",
        action: "omit",
        reason: "local resolver topology",
    },
    ProjectionFieldSpec {
        field: "provider.direct_last_prompt",
        classification: "secret",
        action: "omit",
        reason: "raw prompt text is not context memory",
    },
    ProjectionFieldSpec {
        field: "provider.direct_last_error",
        classification: "local_only",
        action: "omit",
        reason: "error text may contain request ids or provider diagnostics",
    },
    ProjectionFieldSpec {
        field: "provider.direct_last_event",
        classification: "local_only",
        action: "omit",
        reason: "free-form event text remains local",
    },
    ProjectionFieldSpec {
        field: "provider.direct_pending_id",
        classification: "local_only",
        action: "omit",
        reason: "local request correlation id",
    },
    ProjectionFieldSpec {
        field: "provider.direct_last_request_id",
        classification: "local_only",
        action: "omit",
        reason: "local request correlation id",
    },
    ProjectionFieldSpec {
        field: "provider.tcp.*",
        classification: "local_only",
        action: "omit",
        reason: "TCP diagnostics include local transport details",
    },
    ProjectionFieldSpec {
        field: "wifi.ssid",
        classification: "secret",
        action: "omit",
        reason: "raw SSID is user-local configuration",
    },
    ProjectionFieldSpec {
        field: "wifi.passphrase",
        classification: "secret",
        action: "omit",
        reason: "raw Wi-Fi passphrase is never exported",
    },
    ProjectionFieldSpec {
        field: "system.boot_log.raw",
        classification: "local_only",
        action: "omit",
        reason: "raw boot log remains behind local-only methods",
    },
    ProjectionFieldSpec {
        field: "current.recovery_lifeline_status",
        classification: "local_only",
        action: "omit",
        reason:
            "recovery lifeline status fact is local current-boot evidence, not provider context",
    },
    ProjectionFieldSpec {
        field: "recovery.lifeline.status.current_boot",
        classification: "local_only",
        action: "omit",
        reason: "local recovery status locator is authority-bearing evidence, not provider context",
    },
    ProjectionFieldSpec {
        field: "boot_log.summary.current",
        classification: "local_only",
        action: "omit",
        reason: "boot-log summary is a local locator, not provider context",
    },
    ProjectionFieldSpec {
        field: "records[].source",
        classification: "local_only",
        action: "omit",
        reason: "source file paths and local method names remain trace-only locators",
    },
];

pub(crate) fn emit_provider_minimal_projection(
    status: &SystemSnapshot,
    provider: &provider::Snapshot,
    event_id: event_log::EventId,
) {
    let trust_positive = provider_trust_positive(provider.trust_state);
    let evidence = provider_context_evidence(status, provider);

    raw_line("        \"schema\": \"raios.provider_context_projection.v0\",");
    raw_line("        \"mode\": \"local_read_only\",");
    raw_line("        \"profile\": \"provider_minimal\",");
    raw_line("        \"provider_export\": \"disabled\",");
    raw_line("        \"redaction_projection\": \"present\",");
    raw_line("        \"classification_default\": \"local_only\",");
    raw_line("        \"unclassified_field_policy\": \"omit\",");
    raw_line("        \"packet_evidence\": {");
    raw_line("          \"canonicalization\": \"raios.provider_minimal.packet.canonical.v0\",");
    raw("          \"projected_packet_hash\": ");
    json_sha256(evidence.projected_packet_hash);
    raw_line(",");
    raw("          \"exported_field_list_hash\": ");
    json_sha256(evidence.exported_field_list_hash);
    raw_line(",");
    raw("          \"omitted_field_list_hash\": ");
    json_sha256(evidence.omitted_field_list_hash);
    raw_line(",");
    raw("          \"redaction_policy_hash\": ");
    json_sha256(evidence.redaction_policy_hash);
    raw_line(",");
    raw("          \"field_classification_hash\": ");
    json_sha256(evidence.field_classification_hash);
    raw_line(",");
    raw("          \"token_budget_hash\": ");
    json_sha256(evidence.token_budget_hash);
    raw_line("");
    raw_line("        },");
    raw("        \"local_projection_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("        \"audit_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("        \"provider_trust_state\": ");
    json_str(provider.trust_state);
    raw_line(",");
    raw("        \"provider_trust_positive\": ");
    raw_bool(trust_positive);
    raw_line(",");
    raw_line("        \"can_export\": false,");
    raw_line("        \"blocked_by\": [");
    if !trust_positive {
        raw("          {\"gate\": \"provider_trust\", \"state\": ");
        json_str(provider.trust_state);
        raw_line(", \"reason\": \"provider_trust_not_positive\"},");
    }
    raw_line("          {\"gate\": \"provider_context_export_audit_binding\", \"state\": \"missing\", \"reason\": \"provider_context_export_audit_binding_missing\"}");
    raw_line("        ],");
    raw_line("        \"included_fields\": [");
    emit_projection_field_specs(PROVIDER_MINIMAL_INCLUDED_FIELDS, 10);
    raw_line("        ],");
    raw_line("        \"omitted_fields\": [");
    emit_projection_field_specs(PROVIDER_MINIMAL_OMITTED_FIELDS, 10);
    raw_line("        ],");
    raw_line("        \"packet\": {");
    emit_provider_minimal_packet(status, provider);
    raw_line("        }");
}

fn emit_provider_minimal_packet(status: &SystemSnapshot, provider: &provider::Snapshot) {
    raw_line("          \"schema\": \"raios.agent_context.v0\",");
    raw_line("          \"purpose\": \"current_boot_provider_context\",");
    raw_line("          \"profile\": \"provider_minimal\",");
    raw_line("          \"scope\": \"current_boot\",");
    raw_line("          \"budget\": {\"target_tokens\": 2000, \"estimated_tokens\": 900},");
    raw_line("          \"authority_order\": [");
    raw_line("            \"current_snapshot\",");
    raw_line("            \"decision\",");
    raw_line("            \"service_state\",");
    raw_line("            \"summary\"");
    raw_line("          ],");
    raw_line("          \"included\": {");
    raw_line("            \"identity\": [\"mem.fact.identity.stage0\"],");
    raw_line("            \"policy\": [\"adr.0001\", \"adr.0004\"],");
    raw_line("            \"current\": [\"snapshot.current.provider_minimal\", \"capabilities.current_boot\", \"service.inventory.current\", \"problem.list.current\"]");
    raw_line("          },");
    raw_line("          \"current\": {");
    raw_line("            \"os\": {\"name\": \"raiOS\", \"product\": \"raiOS\", \"stage\": \"stage-0\"},");
    raw_line("            \"status\": {");
    emit_status_state_at("framebuffer", status.framebuffer.state, true, 14);
    emit_status_state_at("entropy", status.entropy.state, true, 14);
    emit_status_state_at("usb_xhci", status.usb_xhci.state, true, 14);
    emit_status_state_at("wifi", status.wifi.state, true, 14);
    emit_status_state_at("network", status.network.state, true, 14);
    emit_status_state_at("input", status.input.state, false, 14);
    raw_line("            },");
    raw_line("            \"provider\": {");
    raw("              \"selected\": ");
    json_str(provider.provider_name);
    raw_line(",");
    raw("              \"route\": ");
    json_str(provider.route.as_str());
    raw_line(",");
    raw("              \"api_key_state\": ");
    json_str(if provider.api_key_set {
        "set"
    } else {
        "missing"
    });
    raw_line(",");
    raw("              \"direct_phase\": ");
    json_str(provider.direct_phase);
    raw_line(",");
    raw("              \"direct_endpoint\": ");
    json_str(provider.direct_endpoint);
    raw_line(",");
    raw("              \"direct_model\": ");
    json_str(provider.direct_model);
    raw_line(",");
    raw("              \"trust_state\": ");
    json_str(provider.trust_state);
    raw_line(",");
    raw("              \"pin_kind\": ");
    json_opt_str(provider.trust_pin_kind);
    raw_line(",");
    raw("              \"pin_id\": ");
    json_opt_str(provider.trust_pin_id);
    raw_line(",");
    raw("              \"pin_slot\": ");
    json_opt_str(provider.trust_pin_slot);
    raw_line(",");
    raw("              \"pin_rotation_policy\": ");
    json_str(provider.trust_pin_rotation_policy);
    raw_line(",");
    raw("              \"pin_rotation_id\": ");
    json_opt_str(provider.trust_pin_rotation_id);
    raw_line(",");
    raw("              \"development_bypass\": ");
    raw_bool(provider.trust_development_bypass);
    raw_line(",");
    raw_line("              \"verifier_decision\": {");
    raw("                \"schema\": ");
    json_str(provider.trust_verifier_decision.schema);
    raw_line(",");
    raw("                \"verifier_id\": ");
    json_str(provider.trust_verifier_decision.verifier_id);
    raw_line(",");
    raw("                \"stage\": ");
    json_str(provider.trust_verifier_decision.stage);
    raw_line(",");
    raw("                \"outcome\": ");
    json_str(provider.trust_verifier_decision.outcome);
    raw_line(",");
    raw("                \"reason\": ");
    json_str(provider.trust_verifier_decision.reason);
    crlf();
    raw("              }");
    crlf();
    raw_line("            },");
    raw_line("            \"services\": [");
    emit_service_ids(14);
    raw_line("            ],");
    raw_line("            \"capabilities\": [");
    emit_capability_ids(14);
    raw_line("            ],");
    raw_line("            \"problems\": [");
    emit_problem_objects(status, provider, 14);
    raw_line("            ]");
    raw_line("          },");
    raw_line("          \"records\": [");
    emit_projection_record(
        "mem.fact.identity.stage0",
        "fact",
        "current_snapshot",
        "public",
        "raiOS Stage-0 identity",
        true,
    );
    emit_projection_record(
        "snapshot.current.provider_minimal",
        "redacted_projection",
        "current_snapshot",
        "public",
        "provider_minimal projection of current status and provider trust",
        true,
    );
    emit_projection_record(
        "capabilities.current_boot",
        "capability_index",
        "current_snapshot",
        "public",
        "observe-only capability posture and denied mutation vocabulary",
        true,
    );
    emit_projection_record(
        "service.inventory.current",
        "service_state",
        "service_state",
        "public",
        "stable current-boot service ids",
        true,
    );
    emit_projection_record(
        "problem.list.current",
        "problem_index",
        "current_snapshot",
        "public",
        "current stable problem ids and severities",
        true,
    );
    emit_projection_record(
        "adr.0001",
        "decision",
        "decision",
        "public",
        "build a raiOS-native agent protocol instead of porting the Codex CLI",
        true,
    );
    emit_projection_record(
        "adr.0004",
        "decision",
        "decision",
        "public",
        "memory uses typed local facts and budgeted task-scoped projections",
        false,
    );
    raw_line("          ],");
    raw_line("          \"omitted\": [");
    emit_projection_omission(
        "system.snapshot.raw",
        "local_only",
        "only the provider_minimal projection of selected snapshot fields is included",
        true,
    );
    emit_projection_omission(
        "system.boot_log.raw",
        "local_only",
        "raw boot log is not included",
        true,
    );
    emit_projection_omission(
        "current.recovery_lifeline_status",
        "local_only",
        "local recovery status fact is not included in provider context",
        true,
    );
    emit_projection_omission(
        "recovery.lifeline.status.current_boot",
        "local_only",
        "local recovery status locator is not included in provider context",
        true,
    );
    emit_projection_omission(
        "unclassified.memory_context",
        "local_only",
        "unclassified context fields are omitted by default",
        false,
    );
    raw_line("          ]");
}

pub(crate) fn emit_provider_context_gate(runtime: ui::RuntimeStatus, request: &str) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let profile = provider_context_export_profile(request);
    let profile_supported = method_eq(profile, "provider_minimal");
    let trust_positive = provider_trust_positive(provider.trust_state);
    let evidence = provider_context_evidence(&status, &provider);
    let event_hashes = evidence.event_hashes();
    let check = event_log::check_provider_context_binding_gate(event_hashes);

    begin_response("provider.context_gate");
    raw_line("      \"schema\": \"raios.provider_context_export_gate_state.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"provider_export\": \"disabled\",");
    raw_line("      \"automatic_context_injection\": \"disabled\",");
    raw_line("      \"context_attached_to_provider_body\": false,");
    raw_line("      \"provider_write\": \"not_attempted\",");
    raw("      \"profile\": ");
    json_str(profile);
    raw_line(",");
    raw("      \"profile_supported\": ");
    raw_bool(profile_supported);
    raw_line(",");
    raw_line("      \"gate_state\": {");
    raw("        \"provider_trust_state\": ");
    json_str(provider.trust_state);
    raw_line(",");
    raw("        \"provider_trust_positive\": ");
    raw_bool(trust_positive);
    raw_line(",");
    raw("        \"provider_request_binding\": ");
    json_str(provider_binding_gate_state(&check, "request"));
    raw_line(",");
    raw("        \"provider_export_audit_binding\": ");
    json_str(provider_binding_gate_state(&check, "export"));
    raw_line(",");
    raw("        \"binding_validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"binding_validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"binding_retained\": ");
    raw_bool(check.retained);
    raw_line(",");
    raw("        \"binding_consumed\": ");
    raw_bool(check.consumed);
    raw_line(",");
    raw_line("        \"satisfies_current_boot_export_gate\": false,");
    raw_line("        \"can_export\": false");
    raw_line("      },");
    raw_line("      \"candidate\": {");
    emit_provider_binding_candidate(&check, 8);
    raw_line("      },");
    raw_line("      \"evidence\": {");
    raw_line(
        "        \"packet_canonicalization\": \"raios.provider_minimal.packet.canonical.v0\",",
    );
    raw("        \"projected_packet_hash\": ");
    json_sha256(evidence.projected_packet_hash);
    raw_line(",");
    raw("        \"exported_field_list_hash\": ");
    json_sha256(evidence.exported_field_list_hash);
    raw_line(",");
    raw("        \"omitted_field_list_hash\": ");
    json_sha256(evidence.omitted_field_list_hash);
    raw_line(",");
    raw("        \"redaction_policy_hash\": ");
    json_sha256(evidence.redaction_policy_hash);
    raw_line(",");
    raw("        \"field_classification_hash\": ");
    json_sha256(evidence.field_classification_hash);
    raw_line(",");
    raw("        \"token_budget_hash\": ");
    json_sha256(evidence.token_budget_hash);
    raw_line(",");
    emit_provider_recovery_status_omission_evidence(8, true);
    raw_line("        \"provider_write_path\": \"disabled\"");
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !profile_supported {
        emit_export_gate(
            &mut wrote,
            "profile",
            "unsupported",
            "provider_minimal_is_the_only_v0_export_profile",
        );
    }
    if !trust_positive {
        emit_export_gate(
            &mut wrote,
            "provider_trust",
            provider.trust_state,
            "provider_trust_not_positive",
        );
    }
    if check.status != "valid" {
        emit_export_gate(
            &mut wrote,
            "provider_binding_consumption",
            check.status,
            check.reason,
        );
    }
    emit_export_gate(
        &mut wrote,
        "provider_write_path",
        "disabled",
        "automatic_context_injection_disabled",
    );
    crlf();
    raw_line("      ]");
    end_response("provider.context_gate");
}

pub(crate) fn emit_provider_context_gate_selftest(runtime: ui::RuntimeStatus, request: &str) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let profile = provider_context_export_profile(request);
    let profile_supported = method_eq(profile, "provider_minimal");
    let evidence = provider_context_evidence(&status, &provider);
    let cases = event_log::provider_context_binding_gate_selftest(evidence.event_hashes());
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("provider.context_gate_selftest");
    raw_line("      \"schema\": \"raios.provider_context_gate_negative_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_provider_request_envelope\": false,");
    raw_line("      \"creates_positive_binding_records\": false,");
    raw_line("      \"provider_export\": \"disabled\",");
    raw_line("      \"automatic_context_injection\": \"disabled\",");
    raw_line("      \"context_attached_to_provider_body\": false,");
    raw_line("      \"provider_write\": \"not_attempted\",");
    raw("      \"profile\": ");
    json_str(profile);
    raw_line(",");
    raw("      \"profile_supported\": ");
    raw_bool(profile_supported);
    raw_line(",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"covered_rejections\": [");
    raw_line("        \"missing_export_audit_binding\",");
    raw_line("        \"stale_dropped_request_binding_event_id\",");
    raw_line("        \"stale_dropped_envelope_event_id\",");
    raw_line("        \"previous_boot_or_unretained_event_id\",");
    raw_line("        \"denial_schema_substitution\",");
    raw_line("        \"positive_record_substitution\",");
    raw_line("        \"request_envelope_wrong_variant\",");
    raw_line("        \"request_envelope_event_id_mismatch\",");
    raw_line("        \"request_id_mismatch\",");
    raw_line("        \"request_body_hash_mismatch\",");
    raw_line("        \"request_envelope_hash_mismatch\",");
    raw_line("        \"request_binding_hash_mismatch\",");
    raw_line("        \"provider_minimal_packet_hash_mismatch\",");
    raw_line("        \"exported_field_list_hash_mismatch\",");
    raw_line("        \"omitted_field_list_hash_mismatch\",");
    raw_line("        \"redaction_policy_hash_mismatch\",");
    raw_line("        \"field_classification_hash_mismatch\",");
    raw_line("        \"token_budget_hash_mismatch\",");
    raw_line("        \"provider_trust_evidence_hash_mismatch\",");
    raw_line("        \"trust_bypass_record\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    let mut case_idx = 0usize;
    while case_idx < cases.len() {
        emit_provider_context_gate_selftest_case(&cases[case_idx], case_idx + 1 != cases.len());
        case_idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"satisfies_current_boot_export_gate\": false,");
    raw_line("      \"can_export\": false");
    end_response("provider.context_gate_selftest");
}

fn emit_provider_context_gate_selftest_case(
    case: &event_log::ProviderBindingGateSelfTestCase,
    comma: bool,
) {
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
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_provider_context_injection_gate(runtime: ui::RuntimeStatus, request: &str) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let profile = provider_context_export_profile(request);
    let profile_supported = method_eq(profile, "provider_minimal");
    let trust_positive = provider_trust_positive(provider.trust_state);
    let projection_present = profile_supported;
    let evidence = provider_context_evidence(&status, &provider);
    let check = event_log::check_provider_context_binding_gate(evidence.event_hashes());
    let injection_check = event_log::check_provider_context_injection_gate(
        evidence.event_hashes(),
        provider.trust_state,
    );

    begin_response("provider.context_injection_gate");
    raw_line("      \"schema\": \"raios.provider_context_injection_gate.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"provider_export\": \"disabled\",");
    raw_line("      \"automatic_context_injection\": \"disabled\",");
    raw_line("      \"context_attached_to_provider_body\": false,");
    raw_line("      \"provider_write\": \"not_attempted\",");
    raw("      \"profile\": ");
    json_str(profile);
    raw_line(",");
    raw("      \"profile_supported\": ");
    raw_bool(profile_supported);
    raw_line(",");
    raw_line("      \"gate_state\": {");
    raw("        \"provider_trust_state\": ");
    json_str(provider.trust_state);
    raw_line(",");
    raw("        \"provider_trust_positive\": ");
    raw_bool(trust_positive);
    raw_line(",");
    raw("        \"redaction_projection\": ");
    json_str(if projection_present {
        "present"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("        \"packet_evidence_binding\": \"present\",");
    raw_line("        \"exported_field_list_binding\": \"present\",");
    raw_line("        \"omitted_field_list_binding\": \"present\",");
    raw("        \"provider_request_binding\": ");
    json_str(provider_binding_gate_state(&check, "request"));
    raw_line(",");
    raw("        \"provider_export_audit_binding\": ");
    json_str(provider_binding_gate_state(&check, "export"));
    raw_line(",");
    raw("        \"binding_validation_status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"binding_validation_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"binding_retained\": ");
    raw_bool(check.retained);
    raw_line(",");
    raw("        \"binding_consumed\": ");
    raw_bool(check.consumed);
    raw_line(",");
    raw_line(
        "        \"final_authorization_schema\": \"raios.provider_context_injection_authorization.v0\",",
    );
    raw("        \"final_authorization\": ");
    json_str(provider_injection_authorization_state(&injection_check));
    raw_line(",");
    raw("        \"final_authorization_validation_status\": ");
    json_str(injection_check.status);
    raw_line(",");
    raw("        \"final_authorization_validation_reason\": ");
    json_str(injection_check.reason);
    raw_line(",");
    raw("        \"final_authorization_event_id\": ");
    json_event_id_option(injection_check.authorization_event_id);
    raw_line(",");
    raw("        \"binding_consumption_event_id\": ");
    json_event_id_option(injection_check.binding_consumption_event_id);
    raw_line(",");
    raw("        \"final_authorization_retained\": ");
    raw_bool(injection_check.retained);
    raw_line(",");
    raw("        \"final_prewrite_body_check\": ");
    json_str(provider_injection_body_check_state(&injection_check));
    raw_line(",");
    raw("        \"satisfies_current_boot_export_gate\": ");
    raw_bool(injection_check.satisfies_current_boot_export_gate);
    raw_line(",");
    raw("        \"can_attach_context\": ");
    raw_bool(injection_check.can_attach_context);
    crlf();
    raw_line("      },");
    raw_line("      \"candidate\": {");
    emit_provider_binding_candidate(&check, 8);
    raw_line("      },");
    raw_line("      \"evidence\": {");
    raw_line(
        "        \"packet_canonicalization\": \"raios.provider_minimal.packet.canonical.v0\",",
    );
    raw("        \"projected_packet_hash\": ");
    json_sha256(evidence.projected_packet_hash);
    raw_line(",");
    raw("        \"exported_field_list_hash\": ");
    json_sha256(evidence.exported_field_list_hash);
    raw_line(",");
    raw("        \"omitted_field_list_hash\": ");
    json_sha256(evidence.omitted_field_list_hash);
    raw_line(",");
    raw("        \"redaction_policy_hash\": ");
    json_sha256(evidence.redaction_policy_hash);
    raw_line(",");
    raw("        \"field_classification_hash\": ");
    json_sha256(evidence.field_classification_hash);
    raw_line(",");
    raw("        \"token_budget_hash\": ");
    json_sha256(evidence.token_budget_hash);
    raw_line(",");
    emit_provider_recovery_status_omission_evidence(8, true);
    raw_line("        \"provider_request_body_attachment\": \"blocked_until_final_authorization\"");
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !profile_supported {
        emit_export_gate(
            &mut wrote,
            "profile",
            "unsupported",
            "provider_minimal_is_the_only_v0_export_profile",
        );
    }
    if !trust_positive {
        emit_export_gate(
            &mut wrote,
            "provider_trust",
            provider.trust_state,
            "provider_trust_not_positive",
        );
    }
    if !projection_present {
        emit_export_gate(
            &mut wrote,
            "redaction_projection",
            "missing",
            "provider_minimal_projection_missing",
        );
    }
    if check.status != "valid" {
        emit_export_gate(
            &mut wrote,
            "provider_binding_consumption",
            check.status,
            check.reason,
        );
    }
    if injection_check.status != "blocked" {
        emit_export_gate(
            &mut wrote,
            "provider_context_injection_authorization",
            injection_check.status,
            injection_check.reason,
        );
    }
    emit_export_gate(
        &mut wrote,
        "provider_write_path",
        "disabled",
        "automatic_context_injection_disabled",
    );
    crlf();
    raw_line("      ],");
    raw_line("      \"required\": [");
    raw_line("        \"positive_provider_trust\",");
    raw_line("        \"provider_trust_evidence_hash\",");
    raw_line("        \"provider_trust_verifier_metadata\",");
    raw_line("        \"raios.provider_context_projection.v0\",");
    raw_line("        \"raios.provider_request_binding.v0\",");
    raw_line("        \"raios.provider_context_export_audit_binding.v0\",");
    raw_line("        \"redaction_policy_hash\",");
    raw_line("        \"field_classification_hash\",");
    raw_line("        \"token_budget_hash\",");
    raw_line("        \"raios.provider_context_binding_consumption.v0\",");
    raw_line("        \"raios.provider_context_injection_authorization.v0\",");
    raw_line("        \"final_prewrite_body_hash_check\"");
    raw_line("      ],");
    raw_line("      \"satisfies_current_boot_export_gate\": false,");
    raw_line("      \"can_export\": false");
    end_response("provider.context_injection_gate");
}

pub(crate) fn emit_provider_context_injection_gate_selftest(
    runtime: ui::RuntimeStatus,
    request: &str,
) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let profile = provider_context_export_profile(request);
    let profile_supported = method_eq(profile, "provider_minimal");
    let evidence = provider_context_evidence(&status, &provider);
    let cases = event_log::provider_context_injection_gate_selftest(evidence.event_hashes());
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("provider.context_injection_gate_selftest");
    raw_line("      \"schema\": \"raios.provider_context_injection_gate_negative_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_provider_request_envelope\": false,");
    raw_line("      \"creates_positive_binding_records\": false,");
    raw_line("      \"creates_final_authorization_records\": false,");
    raw_line("      \"provider_export\": \"disabled\",");
    raw_line("      \"automatic_context_injection\": \"disabled\",");
    raw_line("      \"context_attached_to_provider_body\": false,");
    raw_line("      \"provider_write\": \"not_attempted\",");
    raw_line("      \"can_attach_context\": false,");
    raw("      \"profile\": ");
    json_str(profile);
    raw_line(",");
    raw("      \"profile_supported\": ");
    raw_bool(profile_supported);
    raw_line(",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"covered_rejections\": [");
    raw_line("        \"missing_final_authorization\",");
    raw_line("        \"stale_dropped_final_authorization_event_id\",");
    raw_line("        \"final_authorization_schema_substitution\",");
    raw_line("        \"substituted_positive_final_authorization_record\",");
    raw_line("        \"final_authorization_omitted_field_list_hash_mismatch\",");
    raw_line("        \"final_authorization_body_hash_mismatch\",");
    raw_line("        \"final_authorization_trust_downgrade\",");
    raw_line("        \"body_attachment_without_final_authorization\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    let mut case_idx = 0usize;
    while case_idx < cases.len() {
        emit_provider_context_injection_gate_selftest_case(
            &cases[case_idx],
            case_idx + 1 != cases.len(),
        );
        case_idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"satisfies_current_boot_export_gate\": false,");
    raw_line("      \"can_export\": false");
    end_response("provider.context_injection_gate_selftest");
}

fn emit_provider_context_injection_gate_selftest_case(
    case: &event_log::ProviderContextInjectionGateSelfTestCase,
    comma: bool,
) {
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
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

struct ProviderExportPublicPacket<'a> {
    included_public_ids: Vec<&'a str>,
    excluded_local_only_count: u64,
    packet_hash: [u8; 32],
}

const PROVIDER_EXPORT_SELFTEST_DESTINATION: &str = "provider.openai.responses";
const PROVIDER_EXPORT_SELFTEST_TRUST_STATE: &str = "pinned_spki_verified";
const PROVIDER_EXPORT_SELFTEST_BUDGET_TOKENS: u64 = 4096;
const PROVIDER_EXPORT_SELFTEST_ESTIMATED_TOKENS: u64 = 256;
const PROVIDER_EXPORT_PUBLIC_FIXTURE_ID: &str =
    "mem.decision.provider_export_public_fixture.current_boot.v0";

fn assemble_provider_export_public_packet<'a>(
    context: &'a crate::agent_protocol::durable_store::DurableMemoryContext,
) -> ProviderExportPublicPacket<'a> {
    let mut included_public_ids = Vec::new();
    let mut records = Vec::new();
    let mut excluded_local_only_count = 0u64;
    let mut idx = 0usize;
    while idx < context.records.len() {
        let record = &context.records[idx];
        if record.exportable {
            included_public_ids.push(record.id.as_str());
            records.push(V::InlineObject(vec![
                f("id", s(record.id.as_str())),
                f("kind", s(record.kind)),
                f("entity", s(record.entity.as_str())),
                f("predicate", s(record.predicate.as_str())),
                f("classification", s(record.classification)),
                f("authority", s(record.authority.as_str())),
                f("scope", s("durable")),
                f("exportable", b(true)),
            ]));
        } else {
            excluded_local_only_count = excluded_local_only_count.saturating_add(1);
        }
        idx += 1;
    }

    let packet_record_count = included_public_ids.len() as u64;
    let canonical_packet = V::Object(vec![
        f("profile", s("provider_minimal")),
        f("scope", s("durable")),
        f("packet_all_records_public", b(true)),
        f("packet_record_count", V::U64(packet_record_count)),
        f("records", V::Array(records)),
    ]);

    ProviderExportPublicPacket {
        included_public_ids,
        excluded_local_only_count,
        packet_hash: sha256_of_json(&canonical_packet),
    }
}

fn included_public_ids_value<'a>(ids: &[&'a str]) -> V<'a> {
    let mut values = Vec::with_capacity(ids.len());
    let mut idx = 0usize;
    while idx < ids.len() {
        values.push(V::Str(ids[idx]));
        idx += 1;
    }
    V::Array(values)
}

pub(crate) fn emit_provider_context_export_packet_selftest(request: &str) {
    let _profile = provider_context_export_profile(request);
    let context = memory_store::durable_memory_context();
    let packet = assemble_provider_export_public_packet(&context);
    let packet_record_count = packet.included_public_ids.len() as u64;

    begin_response("provider.context_export_packet_selftest");
    emit_record_fields(
        vec![
            f("test_infrastructure", b(true)),
            f("gate_evaluated", b(false)),
            f("authorized", b(false)),
            f("export_performed", b(false)),
            f("provider_write", s("not_attempted")),
            f("packet_all_records_public", b(true)),
            f("packet_record_count", V::U64(packet_record_count)),
            f("packet_hash", V::Sha256(packet.packet_hash)),
            f(
                "included_public_ids",
                included_public_ids_value(&packet.included_public_ids),
            ),
            f(
                "excluded_local_only_count",
                V::U64(packet.excluded_local_only_count),
            ),
            f("owner_sealed", b(false)),
            f("trust_tier", s("dev_key_not_owner_sealed")),
        ],
        6,
    );
    end_response("provider.context_export_packet_selftest");
}

pub(crate) fn emit_provider_context_export_authorized_selftest(request: &str) {
    let _profile = provider_context_export_profile(request);
    let context = memory_store::durable_memory_context();
    let packet = assemble_provider_export_public_packet(&context);
    let packet_record_count = packet.included_public_ids.len() as u64;
    if !provider_export_packet_contains_id(
        &packet.included_public_ids,
        PROVIDER_EXPORT_PUBLIC_FIXTURE_ID,
    ) {
        begin_response("provider.context_export_authorized_selftest");
        emit_record_fields(
            provider_export_authorized_selftest_denied_fields(
                "provider.context_export_authorized_selftest",
                &packet,
                packet_record_count,
                "public_fixture_absent",
            ),
            6,
        );
        end_response("provider.context_export_authorized_selftest");
        return;
    }

    let audit_binding_hash = provider_export_selftest_audit_binding_hash(packet.packet_hash);
    let gate_input = provider_export_authorized_selftest_input(
        packet.packet_hash,
        packet_record_count,
        true,
        audit_binding_hash,
    );
    let gate_decision =
        raios_core::scoped_provider_export::evaluate_provider_export_gate(&gate_input);
    let durable_export_audit = if gate_decision.performed {
        Some(memory_store::record_provider_export_authorized_audit(
            packet.packet_hash,
            packet_record_count,
            PROVIDER_EXPORT_SELFTEST_DESTINATION,
            PROVIDER_EXPORT_SELFTEST_TRUST_STATE,
            PROVIDER_EXPORT_SELFTEST_BUDGET_TOKENS,
            audit_binding_hash,
        ))
    } else {
        None
    };

    let mut fields = provider_export_authorized_selftest_fields(
        "provider.context_export_authorized_selftest",
        &packet,
        packet_record_count,
        true,
        gate_decision,
        audit_binding_hash,
    );
    if let Some(evidence) = durable_export_audit.as_ref() {
        fields.push(f(
            "durable_export_audit",
            provider_export_durable_audit_value(evidence),
        ));
    }

    begin_response("provider.context_export_authorized_selftest");
    emit_record_fields(fields, 6);
    end_response("provider.context_export_authorized_selftest");
}

pub(crate) fn emit_provider_context_export_authorized_selftest_smuggle(request: &str) {
    let _profile = provider_context_export_profile(request);
    let context = memory_store::durable_memory_context();
    let packet = assemble_provider_export_public_packet(&context);
    let packet_record_count = (packet.included_public_ids.len() as u64).saturating_add(1);
    let audit_binding_hash = provider_export_selftest_audit_binding_hash(packet.packet_hash);
    let gate_input = provider_export_authorized_selftest_input(
        packet.packet_hash,
        packet_record_count,
        false,
        audit_binding_hash,
    );
    let gate_decision =
        raios_core::scoped_provider_export::evaluate_provider_export_gate(&gate_input);

    let fields = provider_export_authorized_selftest_fields(
        "provider.context_export_authorized_selftest_smuggle",
        &packet,
        packet_record_count,
        false,
        gate_decision,
        audit_binding_hash,
    );

    begin_response("provider.context_export_authorized_selftest_smuggle");
    emit_record_fields(fields, 6);
    end_response("provider.context_export_authorized_selftest_smuggle");
}

fn provider_export_packet_contains_id(ids: &[&str], wanted: &str) -> bool {
    let mut idx = 0usize;
    while idx < ids.len() {
        if ids[idx] == wanted {
            return true;
        }
        idx += 1;
    }
    false
}

fn provider_export_selftest_audit_binding_hash(packet_hash: [u8; 32]) -> [u8; 32] {
    sha256_of_json(&V::Object(vec![
        f("packet_hash", V::Sha256(packet_hash)),
        f("destination", s(PROVIDER_EXPORT_SELFTEST_DESTINATION)),
        f("trust_state", s(PROVIDER_EXPORT_SELFTEST_TRUST_STATE)),
        f(
            "budget_tokens",
            V::U64(PROVIDER_EXPORT_SELFTEST_BUDGET_TOKENS),
        ),
    ]))
}

fn provider_export_authorized_selftest_input(
    packet_hash: [u8; 32],
    packet_record_count: u64,
    packet_all_records_public: bool,
    audit_binding_hash: [u8; 32],
) -> raios_core::scoped_provider_export::ProviderExportGateInput<'static> {
    raios_core::scoped_provider_export::ProviderExportGateInput {
        method: Some("provider.context_export"),
        profile: Some("provider_minimal"),
        trust_state: Some(PROVIDER_EXPORT_SELFTEST_TRUST_STATE),
        tls_certificate_verification_bypassed: false,
        packet_all_records_public,
        packet_record_count: Some(packet_record_count),
        budget_tokens: Some(PROVIDER_EXPORT_SELFTEST_BUDGET_TOKENS),
        packet_estimated_tokens: Some(PROVIDER_EXPORT_SELFTEST_ESTIMATED_TOKENS),
        audit_packet_hash: Some(packet_hash),
        audit_destination: Some(PROVIDER_EXPORT_SELFTEST_DESTINATION),
        audit_trust_snapshot_present: true,
        audit_binding_hash: Some(audit_binding_hash),
    }
}

fn provider_export_authorized_selftest_denied_fields<'a>(
    method: &'static str,
    packet: &'a ProviderExportPublicPacket<'a>,
    packet_record_count: u64,
    reason: &'static str,
) -> Vec<raios_core::record::Field<'a>> {
    vec![
        f("method", s(method)),
        f("test_infrastructure", b(true)),
        f("gate_evaluated", b(false)),
        f("authorized", b(false)),
        f("gate_status", s("denied")),
        f("gate_reason", s(reason)),
        f("owner_sealed", b(false)),
        f("persistence_claimed", b(false)),
        f("trust_tier", s("dev_key_not_owner_sealed")),
        f("export_performed", b(false)),
        f("transmission_performed", b(false)),
        f("provider_write", s("selftest_no_transmission")),
        f("method_under_test", s("provider.context_export")),
        f("profile", s("provider_minimal")),
        f("packet_all_records_public", b(true)),
        f("public_fixture_present", b(false)),
        f("packet_record_count", V::U64(packet_record_count)),
        f("packet_hash", V::Sha256(packet.packet_hash)),
        f(
            "included_public_ids",
            included_public_ids_value(&packet.included_public_ids),
        ),
        f(
            "excluded_local_only_count",
            V::U64(packet.excluded_local_only_count),
        ),
    ]
}

fn provider_export_authorized_selftest_fields<'a>(
    method: &'static str,
    packet: &'a ProviderExportPublicPacket<'a>,
    packet_record_count: u64,
    packet_all_records_public: bool,
    gate_decision: raios_core::scoped_provider_export::ProviderExportGateDecision,
    audit_binding_hash: [u8; 32],
) -> Vec<raios_core::record::Field<'a>> {
    vec![
        f("method", s(method)),
        f("test_infrastructure", b(true)),
        f("gate_evaluated", b(true)),
        f("authorized", b(gate_decision.performed)),
        f("gate_status", s(gate_decision.status)),
        f("gate_reason", s(gate_decision.reason)),
        f("owner_sealed", b(false)),
        f("persistence_claimed", b(false)),
        f("trust_tier", s("dev_key_not_owner_sealed")),
        f("export_performed", b(false)),
        f("transmission_performed", b(false)),
        f("provider_write", s("selftest_no_transmission")),
        f("method_under_test", s("provider.context_export")),
        f("profile", s("provider_minimal")),
        f("trust_state", s(PROVIDER_EXPORT_SELFTEST_TRUST_STATE)),
        f("destination", s(PROVIDER_EXPORT_SELFTEST_DESTINATION)),
        f(
            "budget_tokens",
            V::U64(PROVIDER_EXPORT_SELFTEST_BUDGET_TOKENS),
        ),
        f(
            "packet_estimated_tokens",
            V::U64(PROVIDER_EXPORT_SELFTEST_ESTIMATED_TOKENS),
        ),
        f("packet_all_records_public", b(packet_all_records_public)),
        f("packet_record_count", V::U64(packet_record_count)),
        f("packet_hash", V::Sha256(packet.packet_hash)),
        f("audit_binding_hash", V::Sha256(audit_binding_hash)),
        f(
            "public_fixture_present",
            b(provider_export_packet_contains_id(
                &packet.included_public_ids,
                PROVIDER_EXPORT_PUBLIC_FIXTURE_ID,
            )),
        ),
        f(
            "included_public_ids",
            included_public_ids_value(&packet.included_public_ids),
        ),
        f(
            "excluded_local_only_count",
            V::U64(packet.excluded_local_only_count),
        ),
    ]
}

fn provider_export_durable_audit_value(
    audit: &memory_store::ProviderExportAuthorizedAuditOutcome,
) -> V<'static> {
    let evidence = audit.evidence.as_ref();
    let duplicate = audit.dedupe == "duplicate_ram_only";
    let durable_append = match evidence {
        Some(evidence) => evidence.durable_append,
        None if duplicate => "not_attempted_deduplicated",
        None => "not_attempted",
    };
    let performed = evidence.map(|evidence| evidence.performed).unwrap_or(false);
    let append_reason = match evidence {
        Some(evidence) => evidence.reason,
        None if duplicate => "deduplicated_first_audit_cited",
        None => audit.dedupe,
    };
    let payload_sha256 = match evidence {
        Some(evidence) => evidence.payload_sha256,
        None => audit.cited_payload_sha256,
    };
    let seq = match evidence {
        Some(evidence) => evidence.seq,
        None => audit.cited_seq,
    };

    V::InlineObject(vec![
        f(
            "gate",
            s(raios_core::scoped_provider_export::SCOPED_PROVIDER_EXPORT_DECISION_ID),
        ),
        f(
            "gate_schema",
            s(raios_core::scoped_provider_export::SCOPED_PROVIDER_EXPORT_DECISION_SCHEMA),
        ),
        f("dedupe", s(audit.dedupe)),
        f("record_id", s(audit.record_id)),
        f(
            "record_schema",
            s(evidence
                .map(|evidence| evidence.record_schema)
                .unwrap_or("raios.memory_record.v0")),
        ),
        f("kind", s("export_audit")),
        f("classification", s("local_only")),
        f(
            "record_authority",
            s(evidence
                .map(|evidence| evidence.record_authority)
                .unwrap_or("core_ledger")),
        ),
        f("supersedes", V::Array(vec![])),
        f("durable_append", s(durable_append)),
        f("performed", b(performed)),
        f("append_reason", s(append_reason)),
        f("payload_sha256", optional_sha256_value(payload_sha256)),
        f(
            "frame_sha256",
            optional_sha256_value(evidence.and_then(|evidence| evidence.frame_sha256)),
        ),
        f(
            "readback_sha256",
            optional_sha256_value(evidence.and_then(|evidence| evidence.readback_sha256)),
        ),
        f(
            "reparse_valid",
            b(evidence
                .map(|evidence| evidence.reparse_valid)
                .unwrap_or(false)),
        ),
        f("seq", optional_u64_value(seq)),
        f(
            "tail_seq_before",
            optional_u64_value(evidence.and_then(|evidence| evidence.tail_seq_before)),
        ),
        f(
            "count_before",
            optional_u64_value(evidence.and_then(|evidence| evidence.count_before)),
        ),
        f(
            "tail_seq_after",
            optional_u64_value(evidence.and_then(|evidence| evidence.tail_seq_after)),
        ),
        f(
            "count_after",
            optional_u64_value(evidence.and_then(|evidence| evidence.count_after)),
        ),
        f(
            "owner_sealed",
            b(evidence
                .map(|evidence| evidence.owner_sealed)
                .unwrap_or(false)),
        ),
        f(
            "persistence_claimed",
            b(evidence
                .map(|evidence| evidence.persistence_claimed)
                .unwrap_or(false)),
        ),
        f(
            "trust_tier",
            s(evidence
                .map(|evidence| evidence.trust_tier)
                .unwrap_or("dev_key_not_owner_sealed")),
        ),
        f("export_performed", b(false)),
        f("provider_write", s("selftest_no_transmission")),
        f("test_infrastructure", b(true)),
    ])
}

fn optional_sha256_value(value: Option<[u8; 32]>) -> V<'static> {
    match value {
        Some(value) => V::Sha256(value),
        None => V::Null,
    }
}

fn optional_u64_value(value: Option<u64>) -> V<'static> {
    match value {
        Some(value) => V::U64(value),
        None => V::Null,
    }
}

pub(crate) fn emit_provider_context_export_denied(
    runtime: ui::RuntimeStatus,
    request: &str,
    denial_event_id: event_log::EventId,
) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let profile = provider_context_export_profile(request);
    let profile_supported = method_eq(profile, "provider_minimal");
    let trust_positive = provider_trust_positive(provider.trust_state);
    let projection_present = profile_supported;
    let evidence = provider_context_evidence(&status, &provider);
    let event_hashes = evidence.event_hashes();
    let (binding_check, binding_consumption_event_id) =
        event_log::consume_provider_context_binding_gate(event_hashes);
    let binding_consumed = binding_check.status == "valid";
    let request_binding_denial_event_id = if binding_consumed {
        None
    } else {
        Some(event_log::record_provider_request_binding_denied(
            event_hashes,
        ))
    };
    let export_denial_audit_event_id =
        event_log::record_provider_context_export_denial_audit(event_hashes);
    let gate_input = raios_core::scoped_provider_export::ProviderExportGateInput {
        method: Some("provider.context_export"),
        profile: Some(profile),
        trust_state: Some(provider.trust_state),
        tls_certificate_verification_bypassed: provider.trust_state
            == "tls_certificate_verification_bypassed",
        packet_all_records_public: false,
        packet_record_count: None,
        budget_tokens: None,
        packet_estimated_tokens: None,
        audit_packet_hash: None,
        audit_destination: None,
        audit_trust_snapshot_present: false,
        audit_binding_hash: None,
    };
    let gate_decision =
        raios_core::scoped_provider_export::evaluate_provider_export_gate(&gate_input);
    let durable_denial_audit = if gate_decision.performed {
        None
    } else {
        Some(memory_store::record_provider_export_denial_audit(
            gate_decision.reason,
            profile,
            provider.trust_state,
        ))
    };

    serial::write_raw_fmt(format_args!(
        "RAIOS_AGENT_BEGIN provider.context_export\r\n"
    ));
    raw_line("{");
    raw_line("  \"v\": \"raios.agent.v0\",");
    raw_line("  \"t\": \"error\",");
    raw_line("  \"id\": \"serial\",");
    raw_line("  \"body\": {");
    raw_line("    \"method\": \"provider.context_export\",");
    raw("    \"event_id\": ");
    json_event_id(denial_event_id);
    raw_line(",");
    raw("    \"audit_event_id\": ");
    json_event_id(export_denial_audit_event_id);
    raw_line(",");
    raw_line("    \"code\": \"capability_denied\",");
    raw_line("    \"schema\": \"raios.provider_context_export.v0\",");
    raw("    \"message\": ");
    json_str("provider context export is denied until positive provider trust, a provider_minimal projection, packet/redaction/classification/budget evidence, provider request binding, and a provider export audit binding exist");
    raw_line(",");
    raw_line("    \"request\": {");
    raw("      \"provider\": ");
    json_str(provider.provider_name);
    raw_line(",");
    raw("      \"route\": ");
    json_str(provider.route.as_str());
    raw_line(",");
    raw("      \"profile\": ");
    json_str(profile);
    raw_line(",");
    raw("      \"profile_supported\": ");
    raw_bool(profile_supported);
    raw_line(",");
    raw_line("      \"context_schema\": \"raios.agent_context.v0\",");
    raw_line("      \"projection_schema\": \"raios.provider_context_projection.v0\",");
    raw_line("      \"export_schema\": \"raios.provider_context_export.v0\",");
    raw_line("      \"requested_capability\": \"cap.provider.context_export\",");
    raw_line("      \"export_scope\": \"single_provider_request\"");
    raw_line("    },");
    raw_line("    \"gate_state\": {");
    raw("      \"provider_trust_state\": ");
    json_str(provider.trust_state);
    raw_line(",");
    raw("      \"provider_trust_positive\": ");
    raw_bool(trust_positive);
    raw_line(",");
    raw("      \"redaction_projection\": ");
    json_str(if projection_present {
        "present"
    } else {
        "missing"
    });
    raw_line(",");
    raw("      \"field_classification\": ");
    json_str(if projection_present {
        "present"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("      \"packet_evidence_binding\": \"present\",");
    raw_line("      \"exported_field_list_binding\": \"present\",");
    raw_line("      \"omitted_field_list_binding\": \"present\",");
    raw("      \"provider_request_binding\": ");
    json_str(provider_binding_gate_state(&binding_check, "request"));
    raw_line(",");
    raw("      \"provider_request_binding_denial\": ");
    json_str(if binding_consumed {
        "not_created_positive_binding_consumed"
    } else {
        "present_denied_not_bound"
    });
    raw_line(",");
    raw("      \"provider_export_audit_binding\": ");
    json_str(provider_binding_gate_state(&binding_check, "export"));
    raw_line(",");
    raw("      \"provider_binding_consumption\": ");
    json_str(if binding_consumed {
        "consumed_for_gate_evaluation"
    } else {
        "not_consumed"
    });
    raw_line(",");
    raw("      \"binding_validation_status\": ");
    json_str(binding_check.status);
    raw_line(",");
    raw("      \"binding_validation_reason\": ");
    json_str(binding_check.reason);
    raw_line(",");
    raw_line("      \"provider_export_denial_audit\": \"present_denied_no_provider_write\",");
    raw_line("      \"provider_write\": \"not_attempted\",");
    raw_line("      \"can_export\": false");
    raw_line("    },");
    if let Some(request_binding_denial_event_id) = request_binding_denial_event_id {
        raw_line("    \"provider_request_binding_denial\": {");
        raw_line("      \"schema\": \"raios.provider_request_binding_denial.v0\",");
        raw("      \"id\": ");
        json_current_boot_id(
            "provider_request_binding_denial.current_boot",
            request_binding_denial_event_id,
        );
        raw_line(",");
        raw("      \"attempted_request_id\": ");
        json_current_boot_id(
            "provider_request_attempt.current_boot",
            request_binding_denial_event_id,
        );
        raw_line(",");
        raw("      \"event_id\": ");
        json_event_id(request_binding_denial_event_id);
        raw_line(",");
        raw_line("      \"status\": \"denied_not_bound\",");
        raw_line("      \"satisfies_export_gate\": false,");
        raw("      \"provider\": ");
        json_str(provider.provider_name);
        raw_line(",");
        raw("      \"route\": ");
        json_str(provider.route.as_str());
        raw_line(",");
        raw("      \"profile\": ");
        json_str(profile);
        raw_line(",");
        raw_line("      \"classification\": \"public\",");
        raw_line("      \"context_schema\": \"raios.agent_context.v0\",");
        raw_line(
            "      \"packet_canonicalization\": \"raios.provider_minimal.packet.canonical.v0\",",
        );
        raw("      \"projected_packet_hash\": ");
        json_sha256(evidence.projected_packet_hash);
        raw_line(",");
        raw("      \"exported_field_list_hash\": ");
        json_sha256(evidence.exported_field_list_hash);
        raw_line(",");
        raw("      \"omitted_field_list_hash\": ");
        json_sha256(evidence.omitted_field_list_hash);
        raw_line(",");
        raw("      \"redaction_policy_hash\": ");
        json_sha256(evidence.redaction_policy_hash);
        raw_line(",");
        raw("      \"field_classification_hash\": ");
        json_sha256(evidence.field_classification_hash);
        raw_line(",");
        raw("      \"token_budget_hash\": ");
        json_sha256(evidence.token_budget_hash);
        raw_line(",");
        raw_line("      \"provider_write\": \"not_attempted\"");
        raw_line("    },");
    } else {
        raw_line("    \"provider_request_binding_denial\": null,");
    }
    raw_line("    \"provider_binding_consumption\": {");
    raw_line("      \"schema\": \"raios.provider_context_binding_consumption.v0\",");
    raw("      \"event_id\": ");
    json_event_id_option(binding_consumption_event_id);
    raw_line(",");
    raw("      \"status\": ");
    json_str(if binding_consumed {
        "consumed_for_gate_evaluation"
    } else {
        "not_consumed"
    });
    raw_line(",");
    raw_line("      \"satisfies_current_boot_export_gate\": false,");
    raw_line("      \"automatic_context_injection\": \"disabled\",");
    raw_line("      \"context_attached_to_provider_body\": false,");
    raw_line("      \"provider_write\": \"not_attempted\",");
    raw("      \"reason\": ");
    json_str(if binding_consumed {
        "provider_binding_consumed_without_body_attachment"
    } else {
        binding_check.reason
    });
    crlf();
    raw_line("    },");
    raw_line("    \"export_denial_audit\": {");
    raw_line("      \"schema\": \"raios.provider_context_export_denial_audit.v0\",");
    raw("      \"id\": ");
    json_current_boot_id(
        "provider_context_export_denial_audit.current_boot",
        export_denial_audit_event_id,
    );
    raw_line(",");
    raw("      \"event_id\": ");
    json_event_id(export_denial_audit_event_id);
    raw_line(",");
    raw_line("      \"status\": \"denied_no_provider_write\",");
    raw_line("      \"satisfies_export_gate\": false,");
    raw_line("      \"classification\": \"public\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"positive_export_authorization\": false,");
    raw("      \"denial_event_id\": ");
    json_event_id(denial_event_id);
    raw_line(",");
    raw_line("      \"provider_write\": \"not_attempted\"");
    raw_line("    },");
    emit_provider_export_durable_denial_audit(&gate_decision, durable_denial_audit.as_ref());
    raw_line("    \"blocked_by\": [");
    let mut wrote = false;
    if !profile_supported {
        emit_export_gate(
            &mut wrote,
            "profile",
            "unsupported",
            "provider_minimal_is_the_only_v0_export_profile",
        );
    }
    if !trust_positive {
        emit_export_gate(
            &mut wrote,
            "provider_trust",
            provider.trust_state,
            "provider_trust_not_positive",
        );
    }
    if !projection_present {
        emit_export_gate(
            &mut wrote,
            "redaction_projection",
            "missing",
            "provider_minimal_projection_missing",
        );
    }
    if !binding_consumed && binding_check.status == "missing" {
        emit_export_gate(
            &mut wrote,
            "provider_request_binding",
            "missing",
            "provider_request_binding_missing",
        );
        emit_export_gate(
            &mut wrote,
            "provider_context_export_audit_binding",
            "missing",
            "provider_context_export_audit_binding_missing",
        );
    } else if !binding_consumed {
        emit_export_gate(
            &mut wrote,
            "provider_binding_consumption",
            binding_check.status,
            binding_check.reason,
        );
    }
    emit_export_gate(
        &mut wrote,
        "provider_write_path",
        "disabled",
        "automatic_context_injection_disabled",
    );
    crlf();
    raw_line("    ],");
    raw_line("    \"required\": [");
    raw_line("      \"positive_provider_trust\",");
    raw_line("      \"provider_trust_evidence_hash\",");
    raw_line("      \"provider_trust_verifier_metadata\",");
    raw_line("      \"raios.provider_context_projection.v0\",");
    raw_line("      \"raios.provider_context_export.v0\",");
    raw_line("      \"projected_packet_hash\",");
    raw_line("      \"exported_field_list_hash\",");
    raw_line("      \"omitted_field_list_hash\",");
    raw_line("      \"redaction_policy_hash\",");
    raw_line("      \"field_classification_hash\",");
    raw_line("      \"token_budget_hash\",");
    raw_line("      \"provider_request_binding\",");
    raw_line("      \"provider_context_export_audit_binding\",");
    raw_line("      \"checked_current_boot_binding_consumption\",");
    raw_line("      \"audit.event.v0\"");
    raw_line("    ],");
    raw_line("    \"evidence\": {");
    raw_line("      \"local_projection_method\": \"memory.context provider_minimal\",");
    raw_line("      \"local_projection_locator\": \"snapshot.current.provider_minimal\",");
    raw_line("      \"packet_canonicalization\": \"raios.provider_minimal.packet.canonical.v0\",");
    raw("      \"projected_packet_hash\": ");
    json_sha256(evidence.projected_packet_hash);
    raw_line(",");
    raw("      \"exported_field_list_hash\": ");
    json_sha256(evidence.exported_field_list_hash);
    raw_line(",");
    raw("      \"omitted_field_list_hash\": ");
    json_sha256(evidence.omitted_field_list_hash);
    raw_line(",");
    raw("      \"redaction_policy_hash\": ");
    json_sha256(evidence.redaction_policy_hash);
    raw_line(",");
    raw("      \"field_classification_hash\": ");
    json_sha256(evidence.field_classification_hash);
    raw_line(",");
    raw("      \"token_budget_hash\": ");
    json_sha256(evidence.token_budget_hash);
    raw_line(",");
    emit_provider_recovery_status_omission_evidence(6, true);
    raw("      \"provider_request_binding_status\": ");
    json_str(provider_binding_gate_state(&binding_check, "request"));
    raw_line(",");
    raw("      \"provider_request_binding_event_id\": ");
    json_event_id_option(binding_check.request_binding_event_id);
    raw_line(",");
    raw("      \"provider_request_binding_denial_id\": ");
    if let Some(request_binding_denial_event_id) = request_binding_denial_event_id {
        json_current_boot_id(
            "provider_request_binding_denial.current_boot",
            request_binding_denial_event_id,
        );
    } else {
        raw("null");
    }
    raw_line(",");
    raw("      \"provider_request_binding_denial_event_id\": ");
    json_event_id_option(request_binding_denial_event_id);
    raw_line(",");
    raw("      \"provider_request_attempt_id\": ");
    if let Some(request_binding_denial_event_id) = request_binding_denial_event_id {
        json_current_boot_id(
            "provider_request_attempt.current_boot",
            request_binding_denial_event_id,
        );
    } else {
        raw("null");
    }
    raw_line(",");
    raw("      \"export_audit_binding_status\": ");
    json_str(provider_binding_gate_state(&binding_check, "export"));
    raw_line(",");
    raw("      \"export_audit_binding_event_id\": ");
    json_event_id_option(binding_check.export_audit_binding_event_id);
    raw_line(",");
    raw("      \"binding_consumption_event_id\": ");
    json_event_id_option(binding_consumption_event_id);
    raw_line(",");
    raw("      \"export_denial_audit_id\": ");
    json_current_boot_id(
        "provider_context_export_denial_audit.current_boot",
        export_denial_audit_event_id,
    );
    raw_line(",");
    raw("      \"export_denial_audit_event_id\": ");
    json_event_id(export_denial_audit_event_id);
    raw_line(",");
    raw_line("      \"export_denial_audit_satisfies_export_gate\": false,");
    raw_line("      \"denial_event_is_export_binding\": false,");
    raw("      \"denial_event_id\": ");
    json_event_id(denial_event_id);
    crlf();
    raw_line("    }");
    raw_line("  }");
    raw_line("}");
    serial::write_raw_fmt(format_args!("RAIOS_AGENT_END provider.context_export\r\n"));
}

fn emit_provider_export_durable_denial_audit(
    gate_decision: &raios_core::scoped_provider_export::ProviderExportGateDecision,
    audit: Option<&memory_store::ProviderExportDenialAuditOutcome>,
) {
    let evidence = audit.and_then(|audit| audit.evidence.as_ref());
    let duplicate = audit
        .map(|audit| audit.dedupe == "duplicate_ram_only")
        .unwrap_or(false);
    let gate_status = if gate_decision.performed {
        "export_authorized_unwired_no_export"
    } else {
        gate_decision.status
    };
    let dedupe = audit
        .map(|audit| audit.dedupe)
        .unwrap_or("not_applicable_no_denial");
    let record_id = audit.map(|audit| audit.record_id).unwrap_or("");
    let durable_append = match evidence {
        Some(evidence) => evidence.durable_append,
        None if duplicate => "not_attempted_deduplicated",
        None => "not_attempted",
    };
    let performed = evidence.map(|evidence| evidence.performed).unwrap_or(false);
    let append_reason = match evidence {
        Some(evidence) => evidence.reason,
        None if duplicate => "deduplicated_first_audit_cited",
        None => dedupe,
    };
    let payload_sha256 = match evidence {
        Some(evidence) => evidence.payload_sha256,
        None => audit.and_then(|audit| audit.cited_payload_sha256),
    };
    let seq = match evidence {
        Some(evidence) => evidence.seq,
        None => audit.and_then(|audit| audit.cited_seq),
    };

    raw_line("    \"durable_denial_audit\": {");
    raw_line("      \"schema\": \"raios.provider_export_denial_durable_audit.v0\",");
    raw("      \"gate\": ");
    json_str(raios_core::scoped_provider_export::SCOPED_PROVIDER_EXPORT_DECISION_ID);
    raw_line(",");
    raw("      \"gate_schema\": ");
    json_str(raios_core::scoped_provider_export::SCOPED_PROVIDER_EXPORT_DECISION_SCHEMA);
    raw_line(",");
    raw("      \"gate_status\": ");
    json_str(gate_status);
    raw_line(",");
    raw("      \"gate_reason\": ");
    json_str(gate_decision.reason);
    raw_line(",");
    raw("      \"dedupe\": ");
    json_str(dedupe);
    raw_line(",");
    raw("      \"record_id\": ");
    json_str(record_id);
    raw_line(",");
    raw_line("      \"record_schema\": \"raios.memory_record.v0\",");
    raw_line("      \"kind\": \"capability_denial\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"record_authority\": \"core_ledger\",");
    raw_line("      \"supersedes\": [],");
    raw("      \"durable_append\": ");
    json_str(durable_append);
    raw_line(",");
    raw("      \"performed\": ");
    raw_bool(performed);
    raw_line(",");
    raw("      \"append_reason\": ");
    json_str(append_reason);
    raw_line(",");
    raw("      \"payload_sha256\": ");
    json_sha256_option(payload_sha256);
    raw_line(",");
    raw("      \"frame_sha256\": ");
    json_sha256_option(evidence.and_then(|evidence| evidence.frame_sha256));
    raw_line(",");
    raw("      \"readback_sha256\": ");
    json_sha256_option(evidence.and_then(|evidence| evidence.readback_sha256));
    raw_line(",");
    raw("      \"reparse_valid\": ");
    raw_bool(
        evidence
            .map(|evidence| evidence.reparse_valid)
            .unwrap_or(false),
    );
    raw_line(",");
    raw("      \"seq\": ");
    emit_optional_u64(seq);
    raw_line(",");
    raw("      \"tail_seq_before\": ");
    emit_optional_u64(evidence.and_then(|evidence| evidence.tail_seq_before));
    raw_line(",");
    raw("      \"count_before\": ");
    emit_optional_u64(evidence.and_then(|evidence| evidence.count_before));
    raw_line(",");
    raw("      \"tail_seq_after\": ");
    emit_optional_u64(evidence.and_then(|evidence| evidence.tail_seq_after));
    raw_line(",");
    raw("      \"count_after\": ");
    emit_optional_u64(evidence.and_then(|evidence| evidence.count_after));
    raw_line(",");
    raw_line("      \"owner_sealed\": false,");
    raw_line("      \"persistence_claimed\": false,");
    raw_line("      \"trust_tier\": \"dev_key_not_owner_sealed\",");
    raw_line("      \"export_performed\": false,");
    raw_line("      \"provider_write\": \"not_attempted\",");
    raw_line("      \"satisfies_export_gate\": false");
    raw_line("    },");
}

fn emit_optional_u64(value: Option<u64>) {
    if let Some(value) = value {
        raw_fmt(format_args!("{}", value));
    } else {
        raw("null");
    }
}

pub(crate) fn emit_provider_context_hashes(hashes: event_log::ProviderContextHashes) {
    raw("{\"packet_canonicalization\": \"raios.provider_minimal.packet.canonical.v0\", \"projected_packet_hash\": ");
    json_sha256(hashes.projected_packet_hash);
    raw(", \"exported_field_list_hash\": ");
    json_sha256(hashes.exported_field_list_hash);
    raw(", \"omitted_field_list_hash\": ");
    json_sha256(hashes.omitted_field_list_hash);
    raw(", \"redaction_policy_hash\": ");
    json_sha256(hashes.redaction_policy_hash);
    raw(", \"field_classification_hash\": ");
    json_sha256(hashes.field_classification_hash);
    raw(", \"token_budget_hash\": ");
    json_sha256(hashes.token_budget_hash);
    raw("}");
}

fn emit_provider_recovery_status_omission_evidence(spaces: usize, comma: bool) {
    indent(spaces);
    raw_line("\"recovery_status_omission\": {");
    indent(spaces + 2);
    raw_line("\"schema\": \"raios.provider_minimal.local_only_omission.v0\",");
    indent(spaces + 2);
    raw_line("\"status\": \"omitted_from_provider_context\",");
    indent(spaces + 2);
    raw_line("\"fact_field\": \"current.recovery_lifeline_status\",");
    indent(spaces + 2);
    raw_line("\"locator\": \"recovery.lifeline.status.current_boot\",");
    indent(spaces + 2);
    raw_line("\"classification\": \"local_only\",");
    indent(spaces + 2);
    raw_line("\"provider_export\": false,");
    indent(spaces + 2);
    raw_line("\"context_attached_to_provider_body\": false,");
    indent(spaces + 2);
    raw_line("\"provider_write\": \"not_attempted\"");
    indent(spaces);
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_projection_record(
    id: &'static str,
    kind: &'static str,
    authority: &'static str,
    classification: &'static str,
    summary: &'static str,
    comma: bool,
) {
    indent(12);
    raw("{\"id\": ");
    json_str(id);
    raw(", \"kind\": ");
    json_str(kind);
    raw(", \"authority\": ");
    json_str(authority);
    raw(", \"classification\": ");
    json_str(classification);
    raw(", \"summary\": ");
    json_str(summary);
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_projection_omission(
    field: &'static str,
    classification: &'static str,
    reason: &'static str,
    comma: bool,
) {
    indent(10);
    raw("{\"field\": ");
    json_str(field);
    raw(", \"classification\": ");
    json_str(classification);
    raw(", \"action\": \"omit\", \"reason\": ");
    json_str(reason);
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_projection_field_specs(fields: &[ProjectionFieldSpec], spaces: usize) {
    let mut idx = 0usize;
    while idx < fields.len() {
        emit_projection_field_spec(&fields[idx], idx + 1 != fields.len(), spaces);
        idx += 1;
    }
}

fn emit_projection_field_spec(spec: &ProjectionFieldSpec, comma: bool, spaces: usize) {
    indent(spaces);
    raw("{\"field\": ");
    json_str(spec.field);
    raw(", \"classification\": ");
    json_str(spec.classification);
    raw(", \"action\": ");
    json_str(spec.action);
    raw(", \"reason\": ");
    json_str(spec.reason);
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

fn provider_binding_gate_state(
    check: &event_log::ProviderBindingGateCheck,
    _kind: &'static str,
) -> &'static str {
    if check.status == "valid" {
        "present_validated"
    } else if check.reason == "binding_already_consumed" {
        "consumed"
    } else if check.retained {
        "rejected"
    } else {
        "missing"
    }
}

fn provider_injection_authorization_state(
    check: &event_log::ProviderContextInjectionGateCheck,
) -> &'static str {
    if check.authorization_event_id.is_none() {
        "missing"
    } else if check.status == "blocked" {
        "present_blocked"
    } else {
        "present_rejected"
    }
}

fn provider_injection_body_check_state(
    check: &event_log::ProviderContextInjectionGateCheck,
) -> &'static str {
    if check.reason == "final_prewrite_body_hash_mismatch" {
        "mismatch"
    } else if check.authorization_event_id.is_none() || check.status != "blocked" {
        "not_attempted"
    } else {
        "matched"
    }
}

fn emit_provider_binding_candidate(check: &event_log::ProviderBindingGateCheck, spaces: usize) {
    indent(spaces);
    raw("\"request_binding_event_id\": ");
    json_event_id_option(check.request_binding_event_id);
    raw_line(",");
    indent(spaces);
    raw("\"export_audit_binding_event_id\": ");
    json_event_id_option(check.export_audit_binding_event_id);
    raw_line(",");
    indent(spaces);
    raw("\"request_envelope_event_id\": ");
    json_event_id_option(check.request_envelope_event_id);
    raw_line(",");
    indent(spaces);
    raw("\"request_id\": ");
    if let Some(binding) = check.export_audit_binding {
        raw_fmt(format_args!("{}", binding.request_id));
    } else if let Some(binding) = check.request_binding {
        raw_fmt(format_args!("{}", binding.request_id));
    } else {
        raw("null");
    }
    raw_line(",");
    indent(spaces);
    raw("\"request_binding_hash\": ");
    if let Some(binding) = check.request_binding {
        json_sha256(binding.request_binding_hash);
    } else if let Some(binding) = check.export_audit_binding {
        json_sha256(binding.request_binding_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    indent(spaces);
    raw("\"export_audit_binding_hash\": ");
    if let Some(binding) = check.export_audit_binding {
        json_sha256(binding.export_audit_binding_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    indent(spaces);
    raw("\"provider_trust_evidence_hash\": ");
    if let Some(binding) = check.export_audit_binding {
        json_sha256(binding.provider_trust_evidence_hash);
    } else if let Some(binding) = check.request_binding {
        json_sha256(binding.provider_trust_evidence_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    indent(spaces);
    raw("\"provider_trust_verifier_id\": ");
    if let Some(binding) = check.export_audit_binding {
        json_str(binding.provider_trust_verifier.id);
    } else if let Some(binding) = check.request_binding {
        json_str(binding.provider_trust_verifier.id);
    } else {
        raw("null");
    }
    raw_line(",");
    indent(spaces);
    raw("\"provider_trust_hostname_policy\": ");
    if let Some(binding) = check.export_audit_binding {
        json_str(binding.provider_trust_verifier.hostname_policy);
    } else if let Some(binding) = check.request_binding {
        json_str(binding.provider_trust_verifier.hostname_policy);
    } else {
        raw("null");
    }
    raw_line(",");
    indent(spaces);
    raw("\"provider_trust_chain_policy\": ");
    if let Some(binding) = check.export_audit_binding {
        json_str(binding.provider_trust_verifier.chain_policy);
    } else if let Some(binding) = check.request_binding {
        json_str(binding.provider_trust_verifier.chain_policy);
    } else {
        raw("null");
    }
    raw_line(",");
    indent(spaces);
    raw("\"provider_trust_time_policy\": ");
    if let Some(binding) = check.export_audit_binding {
        json_str(binding.provider_trust_verifier.time_policy);
    } else if let Some(binding) = check.request_binding {
        json_str(binding.provider_trust_verifier.time_policy);
    } else {
        raw("null");
    }
    raw_line(",");
    indent(spaces);
    raw("\"retained\": ");
    raw_bool(check.retained);
    raw_line(",");
    indent(spaces);
    raw("\"consumed\": ");
    raw_bool(check.consumed);
    crlf();
}

fn provider_context_evidence(
    status: &SystemSnapshot,
    provider: &provider::Snapshot,
) -> ProviderContextEvidence {
    ProviderContextEvidence {
        projected_packet_hash: provider_minimal_packet_hash(status, provider),
        exported_field_list_hash: projection_field_list_hash(
            "raios.provider_minimal.exported_fields.canonical.v0",
            PROVIDER_MINIMAL_INCLUDED_FIELDS,
        ),
        omitted_field_list_hash: projection_field_list_hash(
            "raios.provider_minimal.omitted_fields.canonical.v0",
            PROVIDER_MINIMAL_OMITTED_FIELDS,
        ),
        redaction_policy_hash: provider_redaction_policy_hash(),
        field_classification_hash: provider_field_classification_hash(),
        token_budget_hash: provider_token_budget_hash(),
    }
}

fn provider_redaction_policy_hash() -> [u8; 32] {
    let mut hash = EvidenceHash::new("raios.provider_minimal.redaction_policy.canonical.v0");
    hash.field("profile", "provider_minimal");
    hash.field("redaction_projection", "present");
    hash.field("unclassified_field_policy", "omit");
    hash.field("secret_field_policy", "omit");
    hash.field("local_only_field_policy", "omit");
    hash.field("provider_write", "not_attempted");
    hash.finish()
}

fn provider_field_classification_hash() -> [u8; 32] {
    let mut hash = EvidenceHash::new("raios.provider_minimal.field_classification.canonical.v0");
    hash_projection_classifications(&mut hash, PROVIDER_MINIMAL_INCLUDED_FIELDS);
    hash_projection_classifications(&mut hash, PROVIDER_MINIMAL_OMITTED_FIELDS);
    hash.finish()
}

fn provider_token_budget_hash() -> [u8; 32] {
    let mut hash = EvidenceHash::new("raios.provider_minimal.token_budget.canonical.v0");
    hash.field("profile", "provider_minimal");
    hash.field("budget.target_tokens", "2000");
    hash.field("budget.estimated_tokens", "900");
    hash.field("budget.enforcement", "bounded_projection");
    hash.finish()
}

fn hash_projection_classifications(hash: &mut EvidenceHash, fields: &[ProjectionFieldSpec]) {
    let mut idx = 0usize;
    while idx < fields.len() {
        hash.field("field", fields[idx].field);
        hash.field("classification", fields[idx].classification);
        hash.field("action", fields[idx].action);
        hash.separator();
        idx += 1;
    }
}

fn provider_minimal_packet_hash(
    status: &SystemSnapshot,
    provider: &provider::Snapshot,
) -> [u8; 32] {
    let mut hash = EvidenceHash::new("raios.provider_minimal.packet.canonical.v0");
    hash.field("schema", "raios.agent_context.v0");
    hash.field("purpose", "current_boot_provider_context");
    hash.field("profile", "provider_minimal");
    hash.field("scope", "current_boot");
    hash.field("budget.target_tokens", "2000");
    hash.field("budget.estimated_tokens", "900");
    hash.array(
        "authority_order",
        &["current_snapshot", "decision", "service_state", "summary"],
    );
    hash.array("included.identity", &["mem.fact.identity.stage0"]);
    hash.array("included.policy", &["adr.0001", "adr.0004"]);
    hash.array(
        "included.current",
        &[
            "snapshot.current.provider_minimal",
            "capabilities.current_boot",
            "service.inventory.current",
            "problem.list.current",
        ],
    );
    hash.field("current.os.name", "raiOS");
    hash.field("current.os.product", "raiOS");
    hash.field("current.os.stage", "stage-0");
    hash_status(&mut hash, status);
    hash_provider(&mut hash, provider);
    hash_services(&mut hash);
    hash_capabilities(&mut hash);
    hash_problems(&mut hash, status, provider);
    hash_projection_records(&mut hash);
    hash.array(
        "packet.omitted",
        &[
            "system.snapshot.raw",
            "system.boot_log.raw",
            "current.recovery_lifeline_status",
            "recovery.lifeline.status.current_boot",
            "unclassified.memory_context",
        ],
    );
    hash.finish()
}

fn projection_field_list_hash(domain: &'static str, fields: &[ProjectionFieldSpec]) -> [u8; 32] {
    let mut hash = EvidenceHash::new(domain);
    let mut idx = 0usize;
    while idx < fields.len() {
        hash.field("field", fields[idx].field);
        hash.field("classification", fields[idx].classification);
        hash.field("action", fields[idx].action);
        hash.field("reason", fields[idx].reason);
        hash.separator();
        idx += 1;
    }
    hash.finish()
}

fn hash_status(hash: &mut EvidenceHash, status: &SystemSnapshot) {
    hash.field(
        "current.status.framebuffer",
        status.framebuffer.state.as_protocol(),
    );
    hash.field("current.status.entropy", status.entropy.state.as_protocol());
    hash.field(
        "current.status.usb_xhci",
        status.usb_xhci.state.as_protocol(),
    );
    hash.field("current.status.wifi", status.wifi.state.as_protocol());
    hash.field("current.status.network", status.network.state.as_protocol());
    hash.field("current.status.input", status.input.state.as_protocol());
}

fn hash_provider(hash: &mut EvidenceHash, provider: &provider::Snapshot) {
    hash.field("current.provider.selected", provider.provider_name);
    hash.field("current.provider.route", provider.route.as_str());
    hash.field(
        "current.provider.api_key_state",
        if provider.api_key_set {
            "set"
        } else {
            "missing"
        },
    );
    hash.field("current.provider.direct_phase", provider.direct_phase);
    hash.field("current.provider.direct_endpoint", provider.direct_endpoint);
    hash.field("current.provider.direct_model", provider.direct_model);
    hash.field("current.provider.trust_state", provider.trust_state);
    hash.opt_field("current.provider.pin_kind", provider.trust_pin_kind);
    hash.opt_field("current.provider.pin_id", provider.trust_pin_id);
    hash.opt_field("current.provider.pin_slot", provider.trust_pin_slot);
    hash.field(
        "current.provider.pin_rotation_policy",
        provider.trust_pin_rotation_policy,
    );
    hash.opt_field(
        "current.provider.pin_rotation_id",
        provider.trust_pin_rotation_id,
    );
    hash.bool_field(
        "current.provider.development_bypass",
        provider.trust_development_bypass,
    );
    hash.field(
        "current.provider.verifier_decision.schema",
        provider.trust_verifier_decision.schema,
    );
    hash.field(
        "current.provider.verifier_decision.verifier_id",
        provider.trust_verifier_decision.verifier_id,
    );
    hash.field(
        "current.provider.verifier_decision.stage",
        provider.trust_verifier_decision.stage,
    );
    hash.field(
        "current.provider.verifier_decision.outcome",
        provider.trust_verifier_decision.outcome,
    );
    hash.field(
        "current.provider.verifier_decision.reason",
        provider.trust_verifier_decision.reason,
    );
}

fn hash_services(hash: &mut EvidenceHash) {
    let mut idx = 0usize;
    while idx < service_inventory::SERVICES.len() {
        hash.field("current.services[]", service_inventory::SERVICES[idx].id);
        idx += 1;
    }
}

fn hash_capabilities(hash: &mut EvidenceHash) {
    let mut idx = 0usize;
    while idx < CAPABILITIES.len() {
        if CAPABILITIES[idx].granted {
            hash.field("current.capabilities[]", CAPABILITIES[idx].id);
        }
        idx += 1;
    }
    hash.field(
        "current.capabilities[]",
        "capability_denied.for_all_mutating_methods",
    );
}

fn hash_problems(hash: &mut EvidenceHash, status: &SystemSnapshot, provider: &provider::Snapshot) {
    let mut wrote = false;
    hash_provider_trust_problem(hash, &mut wrote, provider);
    if !provider.api_key_set {
        hash_problem(
            hash,
            &mut wrote,
            "provider.openai.api_key_missing",
            "info",
            "OpenAI direct requests need a RAM-only API key",
        );
    }
    hash_status_problem(
        hash,
        &mut wrote,
        "framebuffer.unavailable",
        "error",
        "Limine framebuffer is unavailable",
        &status.framebuffer,
    );
    hash_status_problem(
        hash,
        &mut wrote,
        "entropy.not_ready",
        "warning",
        "Entropy is not ready yet",
        &status.entropy,
    );
    hash_status_problem(
        hash,
        &mut wrote,
        "usb_xhci.unavailable",
        "warning",
        "xHCI USB path is missing or degraded",
        &status.usb_xhci,
    );
    hash_status_problem(
        hash,
        &mut wrote,
        "network.unavailable",
        "warning",
        "e1000/IPv4 network path is not configured",
        &status.network,
    );
    hash_status_problem(
        hash,
        &mut wrote,
        "input.unavailable",
        "warning",
        "keyboard or pointer input is missing",
        &status.input,
    );
    match wifi::snapshot().state {
        wifi::WifiState::Detected => hash_problem(
            hash,
            &mut wrote,
            "wifi.avastar.firmware_todo",
            "info",
            "Marvell AVASTAR target is detected, but firmware upload and WPA are not implemented",
        ),
        wifi::WifiState::Missing => hash_problem(
            hash,
            &mut wrote,
            "wifi.avastar.target_absent",
            "info",
            "Surface Pro 4 Marvell AVASTAR Wi-Fi target is not present in this machine profile",
        ),
        wifi::WifiState::NotProbed => {}
    }
    if !wrote {
        hash_problem(
            hash,
            &mut wrote,
            "none",
            "info",
            "no known protocol problems reported",
        );
    }
}

fn hash_provider_trust_problem(
    hash: &mut EvidenceHash,
    wrote: &mut bool,
    provider: &provider::Snapshot,
) {
    let (id, summary) = match provider.trust_state {
        "unknown" => (
            "provider.tls_unknown",
            "OpenAI provider trust has not been established",
        ),
        "tls_certificate_verification_bypassed" => (
            "provider.tls_unverified",
            "OpenAI direct transport is using an explicit unverified TLS development override",
        ),
        "pin_config_missing" => (
            "provider.tls_pin_config_missing",
            "OpenAI direct transport is fail-closed until a provider pin is configured",
        ),
        "pin_config_invalid" => (
            "provider.tls_pin_config_invalid",
            "Configured OpenAI provider pin is invalid",
        ),
        "pin_verifier_unavailable" => (
            "provider.tls_pin_verifier_unavailable",
            "Configured OpenAI provider pin cannot be checked until TLS verifier input access exists",
        ),
        "pin_mismatch" => (
            "provider.tls_pin_mismatch",
            "OpenAI provider certificate did not match the configured pin",
        ),
        "pinned_cert_verified" | "pinned_spki_verified" | "webpki_verified" => return,
        _ => (
            "provider.tls_unknown",
            "OpenAI provider trust state is not recognized by this protocol build",
        ),
    };
    hash_problem(hash, wrote, id, "high", summary);
}

fn hash_status_problem(
    hash: &mut EvidenceHash,
    wrote: &mut bool,
    id: &'static str,
    severity: &'static str,
    summary: &'static str,
    line: &system_status::StatusLine,
) {
    if matches!(
        line.state,
        RowState::Ready | RowState::Configured | RowState::Detected
    ) {
        return;
    }
    hash_problem(hash, wrote, id, severity, summary);
}

fn hash_problem(
    hash: &mut EvidenceHash,
    wrote: &mut bool,
    id: &'static str,
    severity: &'static str,
    summary: &'static str,
) {
    hash.field("current.problems[].id", id);
    hash.field("current.problems[].severity", severity);
    hash.field("current.problems[].summary", summary);
    hash.separator();
    *wrote = true;
}

fn hash_projection_records(hash: &mut EvidenceHash) {
    hash_projection_record(
        hash,
        "mem.fact.identity.stage0",
        "fact",
        "current_snapshot",
        "public",
        "raiOS Stage-0 identity",
    );
    hash_projection_record(
        hash,
        "snapshot.current.provider_minimal",
        "redacted_projection",
        "current_snapshot",
        "public",
        "provider_minimal projection of current status and provider trust",
    );
    hash_projection_record(
        hash,
        "capabilities.current_boot",
        "capability_index",
        "current_snapshot",
        "public",
        "observe-only capability posture and denied mutation vocabulary",
    );
    hash_projection_record(
        hash,
        "service.inventory.current",
        "service_state",
        "service_state",
        "public",
        "stable current-boot service ids",
    );
    hash_projection_record(
        hash,
        "problem.list.current",
        "problem_index",
        "current_snapshot",
        "public",
        "current stable problem ids and severities",
    );
    hash_projection_record(
        hash,
        "adr.0001",
        "decision",
        "decision",
        "public",
        "build a raiOS-native agent protocol instead of porting the Codex CLI",
    );
    hash_projection_record(
        hash,
        "adr.0004",
        "decision",
        "decision",
        "public",
        "memory uses typed local facts and budgeted task-scoped projections",
    );
}

fn hash_projection_record(
    hash: &mut EvidenceHash,
    id: &'static str,
    kind: &'static str,
    authority: &'static str,
    classification: &'static str,
    summary: &'static str,
) {
    hash.field("records[].id", id);
    hash.field("records[].kind", kind);
    hash.field("records[].authority", authority);
    hash.field("records[].classification", classification);
    hash.field("records[].summary", summary);
    hash.separator();
}

struct EvidenceHash {
    hasher: Sha256,
}

impl EvidenceHash {
    fn new(domain: &'static str) -> Self {
        let mut value = Self {
            hasher: Sha256::new(),
        };
        value.field("domain", domain);
        value
    }

    fn field(&mut self, name: &str, value: &str) {
        self.hasher.update(name.as_bytes());
        self.hasher.update(b"=");
        self.hasher.update(value.as_bytes());
        self.hasher.update(b"\n");
    }

    fn bool_field(&mut self, name: &str, value: bool) {
        self.field(name, if value { "true" } else { "false" });
    }

    fn opt_field(&mut self, name: &str, value: Option<&str>) {
        self.field(name, value.unwrap_or("null"));
    }

    fn array(&mut self, name: &str, values: &[&str]) {
        let mut idx = 0usize;
        while idx < values.len() {
            self.field(name, values[idx]);
            idx += 1;
        }
        self.separator();
    }

    fn separator(&mut self) {
        self.hasher.update(b"--\n");
    }

    fn finish(self) -> [u8; 32] {
        self.hasher.finalize().into()
    }
}

pub(crate) fn provider_context_export_method(method: &str) -> bool {
    method_head_eq(method, "provider.context_export")
        || method_head_eq(method, "provider.export_context")
}

pub(crate) fn provider_context_export_profile(method: &str) -> &'static str {
    let arg = provider_context_export_arg(method);
    if method_eq(arg, "provider_minimal") || arg.is_empty() {
        "provider_minimal"
    } else {
        "unsupported"
    }
}

fn provider_trust_positive(trust_state: &str) -> bool {
    matches!(
        trust_state,
        "pinned_cert_verified" | "pinned_spki_verified" | "webpki_verified"
    )
}

pub(crate) fn provider_context_block_reason(trust_state: &str) -> &'static str {
    if provider_trust_positive(trust_state) {
        "provider_context_export_audit_binding_missing"
    } else {
        "provider_trust_not_positive"
    }
}

fn provider_context_export_arg(method: &str) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, "provider.context_export") {
        "provider.context_export".len()
    } else if method_head_eq(method, "provider.export_context") {
        "provider.export_context".len()
    } else if method_head_eq(method, "provider.context_gate") {
        "provider.context_gate".len()
    } else if method_head_eq(method, "provider.context_export_status") {
        "provider.context_export_status".len()
    } else if method_head_eq(method, "provider.context_gate_selftest") {
        "provider.context_gate_selftest".len()
    } else if method_head_eq(method, "provider.context_injection_gate") {
        "provider.context_injection_gate".len()
    } else if method_head_eq(method, "provider.context_injection_gate_selftest") {
        "provider.context_injection_gate_selftest".len()
    } else if method_head_eq(method, "provider.context_export_packet_selftest") {
        "provider.context_export_packet_selftest".len()
    } else if method_head_eq(method, "provider.context_export_authorized_selftest") {
        "provider.context_export_authorized_selftest".len()
    } else if method_head_eq(
        method,
        "provider.context_export_authorized_selftest_smuggle",
    ) {
        "provider.context_export_authorized_selftest_smuggle".len()
    } else {
        return "";
    };
    method[head_len..].trim()
}
