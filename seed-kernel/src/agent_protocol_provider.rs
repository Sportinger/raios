use crate::{
    agent_protocol_support::{
        begin_response, crlf, emit_record_fields, emit_record_value_fragment, end_response,
        method_eq, raw_fmt, record_bool as b, record_field as f, record_str as s,
    },
    agent_protocol_system::CAPABILITIES,
    event_log, hello_service, memory_store, personal_shell_service, provider, provider_trust,
    service_inventory, system_problem_facts, system_status,
    system_status::{RowState, SystemSnapshot},
    ui, wifi,
};
use alloc::{vec, vec::Vec};
use core::sync::atomic::{AtomicU64, Ordering};
pub use raios_core::provider_context::{
    projection_field_list_hash, provider_context_block_reason, provider_context_export_arg,
    provider_context_export_method, provider_context_export_profile, provider_context_hashes,
    provider_export_packet_contains_id, provider_export_packet_hash, provider_export_packet_value,
    provider_field_classification_hash, provider_projection_packet_hash,
    provider_redaction_policy_hash, provider_token_budget_hash, provider_trust_positive,
    ProjectionFieldSpec, ProviderContextHashes, ProviderProjectionField, ProviderProjectionInput,
    ProviderProjectionRecord, PROVIDER_MINIMAL_INCLUDED_FIELDS, PROVIDER_MINIMAL_OMITTED_FIELDS,
};
use raios_core::{
    evidence_response::{self as ev, Blocked, Envelope, Evidence},
    provider_projection::{
        project_export_gate, project_provider_minimal, project_trust_honesty, projection_value,
        provider_projection_binding, KeyPresence, ProviderGateInput, ProviderMinimalInput,
        ProviderProjection, ProviderProjectionHashes, ProviderTrustInput,
    },
    provider_trust_descriptor::{
        evaluate_provider_trust_descriptor_honesty, ProviderTrustDescriptor,
    },
    record::{sha256_of_json, Value as V},
    scoped_provider_trust_honesty::ProviderTrustHonestyDecision,
};

static NEXT_PROVIDER_RESPONSE_ID: AtomicU64 = AtomicU64::new(1);

#[rustfmt::skip]
fn emit_provider_v1_response(method: &'static str, family: &'static str,
    projection: ProviderProjection<'_>) {
    let envelope = Envelope {
        response_sequence: NEXT_PROVIDER_RESPONSE_ID.fetch_add(1, Ordering::Relaxed),
        family,
        scope: "current_boot",
        classification: projection.classification.as_str(),
        source_method: method,
        event_sequence: None,
        facts: projection.facts,
        evidence: projection.evidence.into_iter().map(ev::evidence_value).collect(),
        decision: projection.decision,
    };
    raw_fmt(format_args!(
        "{} {method}\r\n",
        concat!("RAIOS_AGENT_", "BEGIN")
    ));
    emit_record_value_fragment(ev::response_value(&envelope), 0);
    crlf();
    raw_fmt(format_args!("RAIOS_AGENT_END {method}\r\n"));
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

pub(crate) struct LiveProviderTrustHonesty {
    pub(crate) descriptor: ProviderTrustDescriptor<'static>,
    pub(crate) descriptor_sha256: [u8; 32],
    pub(crate) decision: ProviderTrustHonestyDecision,
}

pub(crate) fn live_provider_trust_honesty() -> LiveProviderTrustHonesty {
    let snap = provider_trust::snapshot();
    let trust_state = snap.state.as_protocol();
    let verifier = snap.verifier;
    let descriptor = ProviderTrustDescriptor {
        provider_id: "openai",
        id: verifier.id,
        host: verifier.host,
        port: verifier.port,
        transport: verifier.transport,
        hostname_policy: verifier.hostname_policy,
        pin_policy: verifier.pin_policy,
        chain_policy: verifier.chain_policy,
        time_policy: verifier.time_policy,
        certificate_verify_policy: verifier.certificate_verify_policy,
        trust_state,
        development_bypass: snap.development_bypass,
        claims_chain_validated: false,
        claims_time_validated: false,
    };
    let descriptor_sha256 = descriptor.record_sha256();
    let decision = evaluate_provider_trust_descriptor_honesty(&descriptor);

    LiveProviderTrustHonesty {
        descriptor,
        descriptor_sha256,
        decision,
    }
}

pub(crate) fn emit_provider_trust_honesty(_request: &str) {
    let honesty = live_provider_trust_honesty();
    let descriptor = honesty.descriptor;
    let decision = honesty.decision;

    emit_provider_v1_response(
        "provider.trust_honesty",
        "provider.trust_honesty",
        project_trust_honesty(ProviderTrustInput {
            provider_id: descriptor.provider_id,
            descriptor_id: descriptor.id,
            host: descriptor.host,
            descriptor_sha256: honesty.descriptor_sha256,
            trust_state: descriptor.trust_state,
            chain_policy: descriptor.chain_policy,
            time_policy: descriptor.time_policy,
            development_bypass: descriptor.development_bypass,
            status: decision.status,
            reason: decision.reason,
            honest: decision.honest,
            chain_validated: decision.chain_validated,
            time_validated: decision.time_validated,
            claims_chain_validated: false,
            claims_time_validated: false,
        }),
    );
}

pub(crate) fn emit_provider_minimal_projection(
    status: &SystemSnapshot,
    provider: &provider::Snapshot,
    event_id: event_log::EventId,
) {
    let services = current_service_ids_value();
    let problems = current_problem_objects_value(status, provider);
    emit_record_value_fragment(
        provider_minimal_projection_value(status, provider, event_id, services, problems),
        8,
    );
}

pub(crate) fn current_service_ids_value() -> V<'static> {
    let hello = hello_service::loaded_snapshot();
    let personal = personal_shell_service::lifecycle_snapshot();
    let mut services = service_inventory::SERVICES
        .iter()
        .map(|service| s(service.id))
        .collect::<Vec<_>>();
    if personal.running {
        services.push(s("svc.user.shell"));
    }
    if hello.is_some() {
        services.push(s(hello_service::SERVICE_ID));
    }
    V::Array(services)
}

pub(crate) fn current_problem_objects_value(
    status: &SystemSnapshot,
    provider: &provider::Snapshot,
) -> V<'static> {
    let mut problems = system_problem_facts::collect(status, provider, wifi::snapshot())
        .map(|problem| {
            V::InlineObject(vec![
                f("id", s(problem.id)),
                f("severity", s(problem.severity.as_protocol())),
                f("summary", s(problem.summary)),
            ])
        })
        .collect::<Vec<_>>();
    if problems.is_empty() {
        problems.push(V::InlineObject(vec![
            f("id", s("none")),
            f("severity", s("info")),
            f("summary", s("no known protocol problems reported")),
        ]));
    }
    V::Array(problems)
}

pub(crate) fn provider_minimal_projection_value(
    status: &SystemSnapshot,
    provider: &provider::Snapshot,
    event_id: event_log::EventId,
    services: V<'static>,
    problems: V<'static>,
) -> V<'static> {
    let trust_positive = provider_trust_positive(provider.trust_state);
    let evidence = provider_context_evidence(status, provider);
    projection_value(project_provider_minimal(ProviderMinimalInput {
        packet: provider_minimal_packet_value(status, provider, services, problems),
        included_fields: projection_field_specs_value(PROVIDER_MINIMAL_INCLUDED_FIELDS),
        omitted_fields: projection_field_specs_value(PROVIDER_MINIMAL_OMITTED_FIELDS),
        hashes: ProviderProjectionHashes {
            projected_packet_hash: evidence.projected_packet_hash,
            exported_field_list_hash: evidence.exported_field_list_hash,
            omitted_field_list_hash: evidence.omitted_field_list_hash,
            redaction_policy_hash: evidence.redaction_policy_hash,
            field_classification_hash: evidence.field_classification_hash,
            token_budget_hash: evidence.token_budget_hash,
        },
        trust_state: provider.trust_state,
        trust_positive,
        key_presence: if provider.api_key_set {
            KeyPresence::Set
        } else {
            KeyPresence::Missing
        },
        source_event_sequence: Some(event_id.sequence()),
    }))
}

fn provider_minimal_packet_value(
    status: &SystemSnapshot,
    provider: &provider::Snapshot,
    services: V<'static>,
    problems: V<'static>,
) -> V<'static> {
    let mut capabilities = CAPABILITIES
        .iter()
        .filter(|capability| capability.granted)
        .map(|capability| s(capability.id))
        .collect::<Vec<_>>();
    capabilities.push(s("capability_denied.for_all_mutating_methods"));

    V::Object(vec![
        f("schema", s("raios.agent_context.v0")),
        f("purpose", s("current_boot_provider_context")),
        f("profile", s("provider_minimal")),
        f("scope", s("current_boot")),
        f(
            "budget",
            V::InlineObject(vec![
                f("target_tokens", V::U64(2000)),
                f("estimated_tokens", V::U64(900)),
            ]),
        ),
        f(
            "authority_order",
            V::Array(vec![
                s("current_snapshot"),
                s("decision"),
                s("service_state"),
                s("summary"),
            ]),
        ),
        f(
            "included",
            V::Object(vec![
                f(
                    "identity",
                    V::InlineArray(vec![s("mem.fact.identity.stage0")]),
                ),
                f("policy", V::InlineArray(vec![s("adr.0001"), s("adr.0004")])),
                f(
                    "current",
                    V::InlineArray(vec![
                        s("snapshot.current.provider_minimal"),
                        s("capabilities.current_boot"),
                        s("service.inventory.current"),
                        s("problem.list.current"),
                    ]),
                ),
            ]),
        ),
        f(
            "current",
            V::Object(vec![
                f(
                    "os",
                    V::InlineObject(vec![
                        f("name", s("raiOS")),
                        f("product", s("raiOS")),
                        f("stage", s("stage-0")),
                    ]),
                ),
                f(
                    "status",
                    V::Object(vec![
                        f("framebuffer", s(status.framebuffer.state.as_protocol())),
                        f("entropy", s(status.entropy.state.as_protocol())),
                        f("usb_xhci", s(status.usb_xhci.state.as_protocol())),
                        f("wifi", s(status.wifi.state.as_protocol())),
                        f("network", s(status.network.state.as_protocol())),
                        f("input", s(status.input.state.as_protocol())),
                    ]),
                ),
                f(
                    "provider",
                    V::Object(vec![
                        f("selected", s(provider.provider_name)),
                        f("route", s(provider.route.as_str())),
                        f(
                            "api_key_state",
                            s(if provider.api_key_set {
                                "set"
                            } else {
                                "missing"
                            }),
                        ),
                        f("direct_phase", s(provider.direct_phase)),
                        f("direct_endpoint", s(provider.direct_endpoint)),
                        f("direct_model", s(provider.direct_model)),
                        f("trust_state", s(provider.trust_state)),
                        f("pin_kind", optional_str(provider.trust_pin_kind)),
                        f("pin_id", optional_str(provider.trust_pin_id)),
                        f("pin_slot", optional_str(provider.trust_pin_slot)),
                        f("pin_rotation_policy", s(provider.trust_pin_rotation_policy)),
                        f(
                            "pin_rotation_id",
                            optional_str(provider.trust_pin_rotation_id),
                        ),
                        f("development_bypass", b(provider.trust_development_bypass)),
                        f(
                            "verifier_decision",
                            V::Object(vec![
                                f("schema", s(provider.trust_verifier_decision.schema)),
                                f(
                                    "verifier_id",
                                    s(provider.trust_verifier_decision.verifier_id),
                                ),
                                f("stage", s(provider.trust_verifier_decision.stage)),
                                f("status", s(provider.trust_verifier_decision.outcome)),
                                f("reason", s(provider.trust_verifier_decision.reason)),
                            ]),
                        ),
                    ]),
                ),
                f("services", services),
                f("capabilities", V::Array(capabilities)),
                f("problems", problems),
            ]),
        ),
        f(
            "records",
            V::Array(vec![
                projection_record(
                    "mem.fact.identity.stage0",
                    "fact",
                    "current_snapshot",
                    "public",
                    "raiOS Stage-0 identity",
                ),
                projection_record(
                    "snapshot.current.provider_minimal",
                    "redacted_projection",
                    "current_snapshot",
                    "public",
                    "provider_minimal projection of current status and provider trust",
                ),
                projection_record(
                    "capabilities.current_boot",
                    "capability_index",
                    "current_snapshot",
                    "public",
                    "observe-only capability posture and denied mutation vocabulary",
                ),
                projection_record(
                    "service.inventory.current",
                    "service_state",
                    "service_state",
                    "public",
                    "stable current-boot service ids",
                ),
                projection_record(
                    "problem.list.current",
                    "problem_index",
                    "current_snapshot",
                    "public",
                    "current stable problem ids and severities",
                ),
                projection_record(
                    "adr.0001",
                    "decision",
                    "decision",
                    "public",
                    "build a raiOS-native agent protocol instead of porting the Codex CLI",
                ),
                projection_record(
                    "adr.0004",
                    "decision",
                    "decision",
                    "public",
                    "memory uses typed local facts and budgeted task-scoped projections",
                ),
            ]),
        ),
        f(
            "omitted",
            V::Array(vec![
                projection_omission(
                    "system.snapshot.raw",
                    "local_only",
                    "only the provider_minimal projection of selected snapshot fields is included",
                ),
                projection_omission(
                    "system.boot_log.raw",
                    "local_only",
                    "raw boot log is not included",
                ),
                projection_omission(
                    "unclassified.memory_context",
                    "local_only",
                    "unclassified context fields are omitted by default",
                ),
            ]),
        ),
    ])
}

fn optional_str(value: Option<&'static str>) -> V<'static> {
    value.map(s).unwrap_or(V::Null)
}

fn projection_field_specs_value(fields: &[ProjectionFieldSpec]) -> V<'static> {
    V::Array(
        fields
            .iter()
            .map(|spec| {
                V::InlineObject(vec![
                    f("field", s(spec.field)),
                    f("classification", s(spec.classification)),
                    f("action", s(spec.action)),
                    f("reason", s(spec.reason)),
                ])
            })
            .collect(),
    )
}

fn projection_record(
    id: &'static str,
    kind: &'static str,
    authority: &'static str,
    classification: &'static str,
    summary: &'static str,
) -> V<'static> {
    V::InlineObject(vec![
        f("id", s(id)),
        f("kind", s(kind)),
        f("authority", s(authority)),
        f("classification", s(classification)),
        f("summary", s(summary)),
    ])
}

fn projection_omission(
    field: &'static str,
    classification: &'static str,
    reason: &'static str,
) -> V<'static> {
    V::InlineObject(vec![
        f("field", s(field)),
        f("classification", s(classification)),
        f("action", s("omit")),
        f("reason", s(reason)),
    ])
}

fn typed_provider_hashes(evidence: &ProviderContextEvidence) -> ProviderProjectionHashes {
    ProviderProjectionHashes {
        projected_packet_hash: evidence.projected_packet_hash,
        exported_field_list_hash: evidence.exported_field_list_hash,
        omitted_field_list_hash: evidence.omitted_field_list_hash,
        redaction_policy_hash: evidence.redaction_policy_hash,
        field_classification_hash: evidence.field_classification_hash,
        token_budget_hash: evidence.token_budget_hash,
    }
}

fn provider_gate_evidence<'a>(
    profile: &'a str,
    profile_supported: bool,
    provider: &provider::Snapshot,
    evidence: &ProviderContextEvidence,
    check: &event_log::ProviderBindingGateCheck,
) -> Vec<Evidence<'a>> {
    let mut projected = vec![
        Evidence {
            id: "provider_profile",
            kind: "provider_profile",
            status: if profile_supported {
                "verified"
            } else {
                "rejected"
            },
            reason: if profile_supported {
                "provider_minimal_profile_supported"
            } else {
                "provider_minimal_is_the_only_v0_export_profile"
            },
            source_event_sequence: None,
            classification: "public",
            facts: V::InlineObject(vec![f("profile", s(profile))]),
        },
        Evidence {
            id: "provider_trust",
            kind: "provider_trust",
            status: if provider_trust_positive(provider.trust_state) {
                "verified"
            } else {
                "rejected"
            },
            reason: if provider_trust_positive(provider.trust_state) {
                "provider_trust_positive"
            } else {
                "provider_trust_not_positive"
            },
            source_event_sequence: None,
            classification: "public",
            facts: V::InlineObject(vec![f("trust_state", s(provider.trust_state))]),
        },
        Evidence {
            id: "key_presence",
            kind: "key_presence",
            status: "observed",
            reason: "key_presence_observed",
            source_event_sequence: None,
            classification: "public",
            facts: V::InlineObject(vec![f(
                "state",
                s(if provider.api_key_set {
                    "set"
                } else {
                    "missing"
                }),
            )]),
        },
        provider_projection_binding(typed_provider_hashes(evidence), None),
        Evidence {
            id: "provider_binding_consumption",
            kind: "provider_binding_consumption",
            status: check.status,
            reason: check.reason,
            source_event_sequence: check
                .request_binding_event_id
                .map(event_log::EventId::sequence),
            classification: "local_only",
            facts: V::InlineObject(vec![
                f("retained", b(check.retained)),
                f("consumed", b(check.consumed)),
                f(
                    "request_binding_event_id",
                    check
                        .request_binding_event_id
                        .map(|id| V::EventSequence(id.sequence()))
                        .unwrap_or(V::Null),
                ),
                f(
                    "export_audit_binding_event_id",
                    check
                        .export_audit_binding_event_id
                        .map(|id| V::EventSequence(id.sequence()))
                        .unwrap_or(V::Null),
                ),
            ]),
        },
    ];
    if let Some(binding) = check.request_binding {
        projected.insert(
            2,
            Evidence {
                id: "provider_trust_binding",
                kind: "provider_trust_binding",
                status: "verified",
                reason: "provider_trust_evidence_bound",
                source_event_sequence: check
                    .request_binding_event_id
                    .map(event_log::EventId::sequence),
                classification: "local_only",
                facts: V::InlineObject(vec![
                    f(
                        "provider_trust_evidence_hash",
                        V::Sha256(binding.provider_trust_evidence_hash),
                    ),
                    f("verifier_id", s(binding.provider_trust_verifier.id)),
                    f(
                        "chain_policy",
                        s(binding.provider_trust_verifier.chain_policy),
                    ),
                    f(
                        "status_detail",
                        s(binding.provider_trust_verifier_decision.outcome),
                    ),
                    f(
                        "status_reason",
                        s(binding.provider_trust_verifier_decision.reason),
                    ),
                ]),
            },
        );
    }
    projected
}

fn provider_gate_blockers<'a>(
    profile_supported: bool,
    trust_state: &'a str,
    check: &event_log::ProviderBindingGateCheck,
) -> Vec<Blocked<'a>> {
    let mut blocked = Vec::new();
    if !profile_supported {
        blocked.push(Blocked {
            evidence_id: "provider_profile",
            status: "rejected",
            reason: "provider_minimal_is_the_only_v0_export_profile",
        });
    }
    if !provider_trust_positive(trust_state) {
        blocked.push(Blocked {
            evidence_id: "provider_trust",
            status: trust_state,
            reason: "provider_trust_not_positive",
        });
    }
    if check.status != "valid" {
        blocked.push(Blocked {
            evidence_id: "provider_binding_consumption",
            status: check.status,
            reason: check.reason,
        });
    }
    blocked
}

fn emit_provider_context_gate_v1(runtime: ui::RuntimeStatus, request: &str) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let profile = provider_context_export_profile(request);
    let profile_supported = method_eq(profile, "provider_minimal");
    let evidence = provider_context_evidence(&status, &provider);
    let check = event_log::check_provider_context_binding_gate(evidence.event_hashes());
    let mut blocked = provider_gate_blockers(profile_supported, provider.trust_state, &check);
    blocked.push(Blocked {
        evidence_id: "provider_write_path",
        status: "disabled",
        reason: "automatic_context_injection_disabled",
    });
    emit_provider_v1_response(
        "provider.context_gate",
        "provider.context_gate",
        project_export_gate(ProviderGateInput {
            facts: V::InlineObject(vec![f("profile", s(profile))]),
            evidence: provider_gate_evidence(
                profile,
                profile_supported,
                &provider,
                &evidence,
                &check,
            ),
            blocked_by: blocked,
            requested_capability: "cap.provider.context_export",
        }),
    );
}

fn emit_provider_context_injection_gate_v1(runtime: ui::RuntimeStatus, request: &str) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let profile = provider_context_export_profile(request);
    let profile_supported = method_eq(profile, "provider_minimal");
    let evidence = provider_context_evidence(&status, &provider);
    let check = event_log::check_provider_context_binding_gate(evidence.event_hashes());
    let injection = event_log::check_provider_context_injection_gate(
        evidence.event_hashes(),
        provider.trust_state,
    );
    let mut projected_evidence =
        provider_gate_evidence(profile, profile_supported, &provider, &evidence, &check);
    projected_evidence.push(Evidence {
        id: "provider_context_injection_authorization",
        kind: "provider_context_injection_authorization",
        status: injection.status,
        reason: injection.reason,
        source_event_sequence: injection
            .authorization_event_id
            .map(event_log::EventId::sequence),
        classification: "local_only",
        facts: V::InlineObject(vec![
            f("retained", b(injection.retained)),
            f(
                "binding_consumption_event_id",
                injection
                    .binding_consumption_event_id
                    .map(|id| V::EventSequence(id.sequence()))
                    .unwrap_or(V::Null),
            ),
        ]),
    });
    projected_evidence.push(Evidence {
        id: "final_prewrite_body_check",
        kind: "provider_body_check",
        status: "not_attempted",
        reason: "final_authorization_missing",
        source_event_sequence: None,
        classification: "local_only",
        facts: V::InlineObject(vec![]),
    });
    let mut blocked = provider_gate_blockers(profile_supported, provider.trust_state, &check);
    blocked.push(Blocked {
        evidence_id: "provider_context_injection_authorization",
        status: injection.status,
        reason: injection.reason,
    });
    blocked.push(Blocked {
        evidence_id: "provider_write_path",
        status: "disabled",
        reason: "automatic_context_injection_disabled",
    });
    emit_provider_v1_response(
        "provider.context_injection_gate",
        "provider.context_injection_gate",
        project_export_gate(ProviderGateInput {
            facts: V::InlineObject(vec![f("profile", s(profile))]),
            evidence: projected_evidence,
            blocked_by: blocked,
            requested_capability: "cap.provider.context_export",
        }),
    );
}

pub(crate) fn emit_provider_context_gate(runtime: ui::RuntimeStatus, request: &str) {
    emit_provider_context_gate_v1(runtime, request);
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

    let case_count = cases.len() as u64;
    let case_values = cases
        .iter()
        .map(provider_context_gate_selftest_case_value)
        .collect();
    begin_response("provider.context_gate_selftest");
    emit_record_fields(
        vec![
            f(
                "schema",
                s("raios.provider_context_gate_negative_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", b(false)),
            f("creates_provider_request_envelope", b(false)),
            f("creates_positive_binding_records", b(false)),
            f("provider_export", s("disabled")),
            f("automatic_context_injection", s("disabled")),
            f("context_attached_to_provider_body", b(false)),
            f("provider_write", s("not_attempted")),
            f("profile", s(profile)),
            f("profile_supported", b(profile_supported)),
            f("case_count", V::U64(case_count)),
            f("passed", b(passed)),
            f(
                "covered_rejections",
                V::Array(vec![
                    s("missing_export_audit_binding"),
                    s("stale_dropped_request_binding_event_id"),
                    s("stale_dropped_envelope_event_id"),
                    s("previous_boot_or_unretained_event_id"),
                    s("denial_schema_substitution"),
                    s("positive_record_substitution"),
                    s("request_envelope_wrong_variant"),
                    s("request_envelope_event_id_mismatch"),
                    s("request_id_mismatch"),
                    s("request_body_hash_mismatch"),
                    s("request_envelope_hash_mismatch"),
                    s("request_binding_hash_mismatch"),
                    s("provider_minimal_packet_hash_mismatch"),
                    s("exported_field_list_hash_mismatch"),
                    s("omitted_field_list_hash_mismatch"),
                    s("redaction_policy_hash_mismatch"),
                    s("field_classification_hash_mismatch"),
                    s("token_budget_hash_mismatch"),
                    s("provider_trust_evidence_hash_mismatch"),
                    s("trust_bypass_record"),
                ]),
            ),
            f("cases", V::Array(case_values)),
            f("satisfies_current_boot_export_gate", b(false)),
            f("can_export", b(false)),
        ],
        6,
    );
    end_response("provider.context_gate_selftest");
}

fn provider_context_gate_selftest_case_value(
    case: &event_log::ProviderBindingGateSelfTestCase,
) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f("passed", b(case.passed)),
    ])
}

pub(crate) fn emit_provider_context_injection_gate(runtime: ui::RuntimeStatus, request: &str) {
    emit_provider_context_injection_gate_v1(runtime, request);
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

    let case_count = cases.len() as u64;
    let case_values = cases
        .iter()
        .map(provider_context_injection_gate_selftest_case_value)
        .collect();
    begin_response("provider.context_injection_gate_selftest");
    emit_record_fields(
        vec![
            f(
                "schema",
                s("raios.provider_context_injection_gate_negative_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", b(false)),
            f("creates_provider_request_envelope", b(false)),
            f("creates_positive_binding_records", b(false)),
            f("creates_final_authorization_records", b(false)),
            f("provider_export", s("disabled")),
            f("automatic_context_injection", s("disabled")),
            f("context_attached_to_provider_body", b(false)),
            f("provider_write", s("not_attempted")),
            f("can_attach_context", b(false)),
            f("profile", s(profile)),
            f("profile_supported", b(profile_supported)),
            f("case_count", V::U64(case_count)),
            f("passed", b(passed)),
            f(
                "covered_rejections",
                V::Array(vec![
                    s("missing_final_authorization"),
                    s("stale_dropped_final_authorization_event_id"),
                    s("final_authorization_schema_substitution"),
                    s("substituted_positive_final_authorization_record"),
                    s("final_authorization_omitted_field_list_hash_mismatch"),
                    s("final_authorization_body_hash_mismatch"),
                    s("final_authorization_trust_downgrade"),
                    s("body_attachment_without_final_authorization"),
                ]),
            ),
            f("cases", V::Array(case_values)),
            f("satisfies_current_boot_export_gate", b(false)),
            f("can_export", b(false)),
        ],
        6,
    );
    end_response("provider.context_injection_gate_selftest");
}

fn provider_context_injection_gate_selftest_case_value(
    case: &event_log::ProviderContextInjectionGateSelfTestCase,
) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f("passed", b(case.passed)),
    ])
}

struct ProviderExportPublicPacket<'a> {
    included_public_ids: Vec<&'a str>,
    records: Vec<ProviderProjectionRecord<'a>>,
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
    let mut records = Vec::with_capacity(context.records.len());
    let mut excluded_local_only_count = 0u64;
    let mut idx = 0usize;
    while idx < context.records.len() {
        let record = &context.records[idx];
        records.push(ProviderProjectionRecord {
            id: record.id.as_str(),
            kind: record.kind,
            entity: record.entity.as_str(),
            predicate: record.predicate.as_str(),
            classification: record.classification,
            authority: record.authority.as_str(),
            exportable: record.exportable,
        });
        if record.exportable {
            included_public_ids.push(record.id.as_str());
        } else {
            excluded_local_only_count = excluded_local_only_count.saturating_add(1);
        }
        idx += 1;
    }

    ProviderExportPublicPacket {
        included_public_ids,
        packet_hash: provider_export_packet_hash(&records),
        records,
        excluded_local_only_count,
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
    if !provider_export_packet_contains_id(&packet.records, PROVIDER_EXPORT_PUBLIC_FIXTURE_ID) {
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
                &packet.records,
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

    let request_binding_denial_id = request_binding_denial_event_id.map(|event_id| {
        alloc::format!(
            "provider_request_binding_denial.current_boot.{:08}",
            event_id.sequence()
        )
    });
    let provider_request_attempt_id = request_binding_denial_event_id.map(|event_id| {
        alloc::format!(
            "provider_request_attempt.current_boot.{:08}",
            event_id.sequence()
        )
    });
    let export_denial_audit_id = alloc::format!(
        "provider_context_export_denial_audit.current_boot.{:08}",
        export_denial_audit_event_id.sequence()
    );
    let provider_request_binding_denial = match request_binding_denial_event_id {
        Some(event_id) => V::Object(vec![
            f("schema", s("raios.provider_request_binding_denial.v0")),
            f(
                "id",
                s(request_binding_denial_id.as_ref().unwrap().as_str()),
            ),
            f(
                "attempted_request_id",
                s(provider_request_attempt_id.as_ref().unwrap().as_str()),
            ),
            f("event_id", V::EventSequence(event_id.sequence())),
            f("status", s("denied_not_bound")),
            f("satisfies_export_gate", b(false)),
            f("provider", s(provider.provider_name)),
            f("route", s(provider.route.as_str())),
            f("profile", s(profile)),
            f("classification", s("public")),
            f("context_schema", s("raios.agent_context.v0")),
            f(
                "packet_canonicalization",
                s("raios.provider_minimal.packet.canonical.v0"),
            ),
            f(
                "projected_packet_hash",
                V::Sha256(evidence.projected_packet_hash),
            ),
            f(
                "exported_field_list_hash",
                V::Sha256(evidence.exported_field_list_hash),
            ),
            f(
                "omitted_field_list_hash",
                V::Sha256(evidence.omitted_field_list_hash),
            ),
            f(
                "redaction_policy_hash",
                V::Sha256(evidence.redaction_policy_hash),
            ),
            f(
                "field_classification_hash",
                V::Sha256(evidence.field_classification_hash),
            ),
            f("token_budget_hash", V::Sha256(evidence.token_budget_hash)),
            f("provider_write", s("not_attempted")),
        ]),
        None => V::Null,
    };

    let mut blocked_by = Vec::new();
    if !profile_supported {
        blocked_by.push(provider_gate_value(
            "profile",
            "unsupported",
            "provider_minimal_is_the_only_v0_export_profile",
        ));
    }
    if !trust_positive {
        blocked_by.push(provider_gate_value(
            "provider_trust",
            provider.trust_state,
            "provider_trust_not_positive",
        ));
    }
    if !projection_present {
        blocked_by.push(provider_gate_value(
            "redaction_projection",
            "missing",
            "provider_minimal_projection_missing",
        ));
    }
    if !binding_consumed && binding_check.status == "missing" {
        blocked_by.push(provider_gate_value(
            "provider_request_binding",
            "missing",
            "provider_request_binding_missing",
        ));
        blocked_by.push(provider_gate_value(
            "provider_context_export_audit_binding",
            "missing",
            "provider_context_export_audit_binding_missing",
        ));
    } else if !binding_consumed {
        blocked_by.push(provider_gate_value(
            "provider_binding_consumption",
            binding_check.status,
            binding_check.reason,
        ));
    }
    blocked_by.push(provider_gate_value(
        "provider_write_path",
        "disabled",
        "automatic_context_injection_disabled",
    ));

    let value = V::Object(vec![
        f("v", s("raios.agent.v0")),
        f("t", s("error")),
        f("id", s("serial")),
        f("body", V::Object(vec![
            f("method", s("provider.context_export")),
            f("event_id", V::EventSequence(denial_event_id.sequence())),
            f("audit_event_id", V::EventSequence(export_denial_audit_event_id.sequence())),
            f("code", s("capability_denied")),
            f("schema", s("raios.provider_context_export.v0")),
            f("message", s("provider context export is denied until positive provider trust, a provider_minimal projection, packet/redaction/classification/budget evidence, provider request binding, and a provider export audit binding exist")),
            f("request", V::Object(vec![
                f("provider", s(provider.provider_name)),
                f("route", s(provider.route.as_str())),
                f("profile", s(profile)),
                f("profile_supported", b(profile_supported)),
                f("context_schema", s("raios.agent_context.v0")),
                f("projection_schema", s("raios.provider_context_projection.v0")),
                f("export_schema", s("raios.provider_context_export.v0")),
                f("requested_capability", s("cap.provider.context_export")),
                f("export_scope", s("single_provider_request")),
            ])),
            f("gate_state", V::Object(vec![
                f("provider_trust_state", s(provider.trust_state)),
                f("provider_trust_positive", b(trust_positive)),
                f("redaction_projection", s(if projection_present { "present" } else { "missing" })),
                f("field_classification", s(if projection_present { "present" } else { "missing" })),
                f("packet_evidence_binding", s("present")),
                f("exported_field_list_binding", s("present")),
                f("omitted_field_list_binding", s("present")),
                f("provider_request_binding", s(provider_binding_gate_state(&binding_check, "request"))),
                f("provider_request_binding_denial", s(if binding_consumed { "not_created_positive_binding_consumed" } else { "present_denied_not_bound" })),
                f("provider_export_audit_binding", s(provider_binding_gate_state(&binding_check, "export"))),
                f("provider_binding_consumption", s(if binding_consumed { "consumed_for_gate_evaluation" } else { "not_consumed" })),
                f("binding_validation_status", s(binding_check.status)),
                f("binding_validation_reason", s(binding_check.reason)),
                f("provider_export_denial_audit", s("present_denied_no_provider_write")),
                f("provider_write", s("not_attempted")),
                f("can_export", b(false)),
            ])),
            f("provider_request_binding_denial", provider_request_binding_denial),
            f("provider_binding_consumption", V::Object(vec![
                f("schema", s("raios.provider_context_binding_consumption.v0")),
                f("event_id", binding_consumption_event_id.map(|id| V::EventSequence(id.sequence())).unwrap_or(V::Null)),
                f("status", s(if binding_consumed { "consumed_for_gate_evaluation" } else { "not_consumed" })),
                f("satisfies_current_boot_export_gate", b(false)),
                f("automatic_context_injection", s("disabled")),
                f("context_attached_to_provider_body", b(false)),
                f("provider_write", s("not_attempted")),
                f("reason", s(if binding_consumed { "provider_binding_consumed_without_body_attachment" } else { binding_check.reason })),
            ])),
            f("export_denial_audit", V::Object(vec![
                f("schema", s("raios.provider_context_export_denial_audit.v0")),
                f("id", s(export_denial_audit_id.as_str())),
                f("event_id", V::EventSequence(export_denial_audit_event_id.sequence())),
                f("status", s("denied_no_provider_write")),
                f("satisfies_export_gate", b(false)),
                f("classification", s("public")),
                f("persistence", s("none")),
                f("positive_export_authorization", b(false)),
                f("denial_event_id", V::EventSequence(denial_event_id.sequence())),
                f("provider_write", s("not_attempted")),
            ])),
            f("durable_denial_audit", provider_export_durable_denial_audit_value(&gate_decision, durable_denial_audit.as_ref())),
            f("blocked_by", V::Array(blocked_by)),
            f("required", V::Array(vec![
                s("positive_provider_trust"),
                s("provider_trust_evidence_hash"),
                s("provider_trust_verifier_metadata"),
                s("raios.provider_context_projection.v0"),
                s("raios.provider_context_export.v0"),
                s("projected_packet_hash"),
                s("exported_field_list_hash"),
                s("omitted_field_list_hash"),
                s("redaction_policy_hash"),
                s("field_classification_hash"),
                s("token_budget_hash"),
                s("provider_request_binding"),
                s("provider_context_export_audit_binding"),
                s("checked_current_boot_binding_consumption"),
                s("audit.event.v0"),
            ])),
            f("evidence", V::Object(vec![
                f("local_projection_method", s("memory.context provider_minimal")),
                f("local_projection_locator", s("snapshot.current.provider_minimal")),
                f("packet_canonicalization", s("raios.provider_minimal.packet.canonical.v0")),
                f("projected_packet_hash", V::Sha256(evidence.projected_packet_hash)),
                f("exported_field_list_hash", V::Sha256(evidence.exported_field_list_hash)),
                f("omitted_field_list_hash", V::Sha256(evidence.omitted_field_list_hash)),
                f("redaction_policy_hash", V::Sha256(evidence.redaction_policy_hash)),
                f("field_classification_hash", V::Sha256(evidence.field_classification_hash)),
                f("token_budget_hash", V::Sha256(evidence.token_budget_hash)),
                f("provider_request_binding_status", s(provider_binding_gate_state(&binding_check, "request"))),
                f("provider_request_binding_event_id", binding_check.request_binding_event_id.map(|id| V::EventSequence(id.sequence())).unwrap_or(V::Null)),
                f("provider_request_binding_denial_id", request_binding_denial_id.as_ref().map(|id| s(id.as_str())).unwrap_or(V::Null)),
                f("provider_request_binding_denial_event_id", request_binding_denial_event_id.map(|id| V::EventSequence(id.sequence())).unwrap_or(V::Null)),
                f("provider_request_attempt_id", provider_request_attempt_id.as_ref().map(|id| s(id.as_str())).unwrap_or(V::Null)),
                f("export_audit_binding_status", s(provider_binding_gate_state(&binding_check, "export"))),
                f("export_audit_binding_event_id", binding_check.export_audit_binding_event_id.map(|id| V::EventSequence(id.sequence())).unwrap_or(V::Null)),
                f("binding_consumption_event_id", binding_consumption_event_id.map(|id| V::EventSequence(id.sequence())).unwrap_or(V::Null)),
                f("export_denial_audit_id", s(export_denial_audit_id.as_str())),
                f("export_denial_audit_event_id", V::EventSequence(export_denial_audit_event_id.sequence())),
                f("export_denial_audit_satisfies_export_gate", b(false)),
                f("denial_event_is_export_binding", b(false)),
                f("denial_event_id", V::EventSequence(denial_event_id.sequence())),
            ])),
        ])),
    ]);

    raw_fmt(format_args!(
        "RAIOS_AGENT_BEGIN provider.context_export\r\n"
    ));
    emit_record_value_fragment(value, 0);
    crlf();
    raw_fmt(format_args!("RAIOS_AGENT_END provider.context_export\r\n"));
}

fn provider_gate_value<'a>(gate: &'a str, state: &'a str, reason: &'a str) -> V<'a> {
    V::InlineObject(vec![
        f("gate", s(gate)),
        f("state", s(state)),
        f("reason", s(reason)),
    ])
}

fn provider_export_durable_denial_audit_value<'a>(
    gate_decision: &'a raios_core::scoped_provider_export::ProviderExportGateDecision,
    audit: Option<&'a memory_store::ProviderExportDenialAuditOutcome>,
) -> V<'a> {
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

    V::Object(vec![
        f("schema", s("raios.provider_export_denial_durable_audit.v0")),
        f(
            "gate",
            s(raios_core::scoped_provider_export::SCOPED_PROVIDER_EXPORT_DECISION_ID),
        ),
        f(
            "gate_schema",
            s(raios_core::scoped_provider_export::SCOPED_PROVIDER_EXPORT_DECISION_SCHEMA),
        ),
        f("gate_status", s(gate_status)),
        f("gate_reason", s(gate_decision.reason)),
        f("dedupe", s(dedupe)),
        f("record_id", s(record_id)),
        f("record_schema", s("raios.memory_record.v0")),
        f("kind", s("capability_denial")),
        f("classification", s("local_only")),
        f("record_authority", s("core_ledger")),
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
        f("owner_sealed", b(false)),
        f("persistence_claimed", b(false)),
        f("trust_tier", s("dev_key_not_owner_sealed")),
        f("export_performed", b(false)),
        f("provider_write", s("not_attempted")),
        f("satisfies_export_gate", b(false)),
    ])
}

pub(crate) fn emit_provider_context_hashes(hashes: event_log::ProviderContextHashes) {
    emit_record_value_fragment(
        V::InlineObject(vec![
            f(
                "packet_canonicalization",
                s("raios.provider_minimal.packet.canonical.v0"),
            ),
            f(
                "projected_packet_hash",
                V::Sha256(hashes.projected_packet_hash),
            ),
            f(
                "exported_field_list_hash",
                V::Sha256(hashes.exported_field_list_hash),
            ),
            f(
                "omitted_field_list_hash",
                V::Sha256(hashes.omitted_field_list_hash),
            ),
            f(
                "redaction_policy_hash",
                V::Sha256(hashes.redaction_policy_hash),
            ),
            f(
                "field_classification_hash",
                V::Sha256(hashes.field_classification_hash),
            ),
            f("token_budget_hash", V::Sha256(hashes.token_budget_hash)),
        ]),
        0,
    );
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

fn provider_minimal_packet_hash(
    status: &SystemSnapshot,
    provider: &provider::Snapshot,
) -> [u8; 32] {
    provider_projection_hashes(status, provider).projected_packet_hash
}

fn provider_projection_hashes(
    status: &SystemSnapshot,
    provider: &provider::Snapshot,
) -> ProviderContextHashes {
    let status_fields = [
        ProviderProjectionField {
            field: "current.status.framebuffer",
            value: status.framebuffer.state.as_protocol(),
        },
        ProviderProjectionField {
            field: "current.status.entropy",
            value: status.entropy.state.as_protocol(),
        },
        ProviderProjectionField {
            field: "current.status.usb_xhci",
            value: status.usb_xhci.state.as_protocol(),
        },
        ProviderProjectionField {
            field: "current.status.wifi",
            value: status.wifi.state.as_protocol(),
        },
        ProviderProjectionField {
            field: "current.status.network",
            value: status.network.state.as_protocol(),
        },
        ProviderProjectionField {
            field: "current.status.input",
            value: status.input.state.as_protocol(),
        },
    ];

    let provider_fields = [
        ProviderProjectionField {
            field: "current.provider.selected",
            value: provider.provider_name,
        },
        ProviderProjectionField {
            field: "current.provider.route",
            value: provider.route.as_str(),
        },
        ProviderProjectionField {
            field: "current.provider.api_key_state",
            value: if provider.api_key_set {
                "set"
            } else {
                "missing"
            },
        },
        ProviderProjectionField {
            field: "current.provider.direct_phase",
            value: provider.direct_phase,
        },
        ProviderProjectionField {
            field: "current.provider.direct_endpoint",
            value: provider.direct_endpoint,
        },
        ProviderProjectionField {
            field: "current.provider.direct_model",
            value: provider.direct_model,
        },
        ProviderProjectionField {
            field: "current.provider.trust_state",
            value: provider.trust_state,
        },
        ProviderProjectionField {
            field: "current.provider.pin_kind",
            value: provider.trust_pin_kind.unwrap_or("null"),
        },
        ProviderProjectionField {
            field: "current.provider.pin_id",
            value: provider.trust_pin_id.unwrap_or("null"),
        },
        ProviderProjectionField {
            field: "current.provider.pin_slot",
            value: provider.trust_pin_slot.unwrap_or("null"),
        },
        ProviderProjectionField {
            field: "current.provider.pin_rotation_policy",
            value: provider.trust_pin_rotation_policy,
        },
        ProviderProjectionField {
            field: "current.provider.pin_rotation_id",
            value: provider.trust_pin_rotation_id.unwrap_or("null"),
        },
        ProviderProjectionField {
            field: "current.provider.development_bypass",
            value: if provider.trust_development_bypass {
                "true"
            } else {
                "false"
            },
        },
        ProviderProjectionField {
            field: "current.provider.verifier_decision.schema",
            value: provider.trust_verifier_decision.schema,
        },
        ProviderProjectionField {
            field: "current.provider.verifier_decision.verifier_id",
            value: provider.trust_verifier_decision.verifier_id,
        },
        ProviderProjectionField {
            field: "current.provider.verifier_decision.stage",
            value: provider.trust_verifier_decision.stage,
        },
        ProviderProjectionField {
            field: "current.provider.verifier_decision.outcome",
            value: provider.trust_verifier_decision.outcome,
        },
        ProviderProjectionField {
            field: "current.provider.verifier_decision.reason",
            value: provider.trust_verifier_decision.reason,
        },
    ];

    let mut services = Vec::with_capacity(service_inventory::SERVICES.len());
    for service in service_inventory::SERVICES {
        services.push(service.id);
    }

    let mut capabilities = Vec::with_capacity(CAPABILITIES.len() + 1);
    for capability in CAPABILITIES {
        if capability.granted {
            capabilities.push(capability.id);
        }
    }
    capabilities.push("capability_denied.for_all_mutating_methods");

    let mut problems = Vec::new();
    let mut wrote_problem = false;
    let mut push_problem = |id: &'static str, severity: &'static str, summary: &'static str| {
        problems.push(ProviderProjectionField {
            field: "current.problems[].id",
            value: id,
        });
        problems.push(ProviderProjectionField {
            field: "current.problems[].severity",
            value: severity,
        });
        problems.push(ProviderProjectionField {
            field: "current.problems[].summary",
            value: summary,
        });
        wrote_problem = true;
    };

    let trust_problem = match provider.trust_state {
        "unknown" => Some((
            "provider.tls_unknown",
            "OpenAI provider trust has not been established",
        )),
        "tls_certificate_verification_bypassed" => Some((
            "provider.tls_unverified",
            "OpenAI direct transport is using an explicit unverified TLS development override",
        )),
        "pin_config_missing" => Some((
            "provider.tls_pin_config_missing",
            "OpenAI direct transport is fail-closed until a provider pin is configured",
        )),
        "pin_config_invalid" => Some((
            "provider.tls_pin_config_invalid",
            "Configured OpenAI provider pin is invalid",
        )),
        "pin_verifier_unavailable" => Some((
            "provider.tls_pin_verifier_unavailable",
            "Configured OpenAI provider pin cannot be checked until TLS verifier input access exists",
        )),
        "pin_mismatch" => Some((
            "provider.tls_pin_mismatch",
            "OpenAI provider certificate did not match the configured pin",
        )),
        "pinned_cert_verified" | "pinned_spki_verified" | "webpki_verified" => None,
        _ => Some((
            "provider.tls_unknown",
            "OpenAI provider trust state is not recognized by this protocol build",
        )),
    };
    if let Some((id, summary)) = trust_problem {
        push_problem(id, "high", summary);
    }
    if !provider.api_key_set {
        push_problem(
            "provider.openai.api_key_missing",
            "info",
            "OpenAI direct requests need a RAM-only API key",
        );
    }

    if !matches!(
        status.framebuffer.state,
        RowState::Ready | RowState::Configured | RowState::Detected
    ) {
        push_problem(
            "framebuffer.unavailable",
            "error",
            "Limine framebuffer is unavailable",
        );
    }
    if !matches!(
        status.entropy.state,
        RowState::Ready | RowState::Configured | RowState::Detected
    ) {
        push_problem("entropy.not_ready", "warning", "Entropy is not ready yet");
    }
    if !matches!(
        status.usb_xhci.state,
        RowState::Ready | RowState::Configured | RowState::Detected
    ) {
        push_problem(
            "usb_xhci.unavailable",
            "warning",
            "xHCI USB path is missing or degraded",
        );
    }
    if !matches!(
        status.network.state,
        RowState::Ready | RowState::Configured | RowState::Detected
    ) {
        push_problem(
            "network.unavailable",
            "warning",
            "e1000/IPv4 network path is not configured",
        );
    }
    if !matches!(
        status.input.state,
        RowState::Ready | RowState::Configured | RowState::Detected
    ) {
        push_problem(
            "input.unavailable",
            "warning",
            "keyboard or pointer input is missing",
        );
    }

    match wifi::snapshot().state {
        wifi::WifiState::Detected => push_problem(
            "wifi.avastar.firmware_todo",
            "info",
            "Marvell AVASTAR target is detected, but firmware upload and WPA are not implemented",
        ),
        wifi::WifiState::Missing => push_problem(
            "wifi.avastar.target_absent",
            "info",
            "Surface Pro 4 Marvell AVASTAR Wi-Fi target is not present in this machine profile",
        ),
        wifi::WifiState::NotProbed => {}
    }
    drop(push_problem);
    if !wrote_problem {
        problems.push(ProviderProjectionField {
            field: "current.problems[].id",
            value: "none",
        });
        problems.push(ProviderProjectionField {
            field: "current.problems[].severity",
            value: "info",
        });
        problems.push(ProviderProjectionField {
            field: "current.problems[].summary",
            value: "no known protocol problems reported",
        });
    }

    let records = [
        ProviderProjectionRecord {
            id: "mem.fact.identity.stage0",
            kind: "fact",
            entity: "",
            predicate: "raiOS Stage-0 identity",
            classification: "public",
            authority: "current_snapshot",
            exportable: true,
        },
        ProviderProjectionRecord {
            id: "snapshot.current.provider_minimal",
            kind: "redacted_projection",
            entity: "",
            predicate: "provider_minimal projection of current status and provider trust",
            classification: "public",
            authority: "current_snapshot",
            exportable: true,
        },
        ProviderProjectionRecord {
            id: "capabilities.current_boot",
            kind: "capability_index",
            entity: "",
            predicate: "observe-only capability posture and denied mutation vocabulary",
            classification: "public",
            authority: "current_snapshot",
            exportable: true,
        },
        ProviderProjectionRecord {
            id: "service.inventory.current",
            kind: "service_state",
            entity: "",
            predicate: "stable current-boot service ids",
            classification: "public",
            authority: "service_state",
            exportable: true,
        },
        ProviderProjectionRecord {
            id: "problem.list.current",
            kind: "problem_index",
            entity: "",
            predicate: "current stable problem ids and severities",
            classification: "public",
            authority: "current_snapshot",
            exportable: true,
        },
        ProviderProjectionRecord {
            id: "adr.0001",
            kind: "decision",
            entity: "",
            predicate: "build a raiOS-native agent protocol instead of porting the Codex CLI",
            classification: "public",
            authority: "decision",
            exportable: true,
        },
        ProviderProjectionRecord {
            id: "adr.0004",
            kind: "decision",
            entity: "",
            predicate: "memory uses typed local facts and budgeted task-scoped projections",
            classification: "public",
            authority: "decision",
            exportable: true,
        },
    ];

    provider_context_hashes(&ProviderProjectionInput {
        status: &status_fields,
        provider: &provider_fields,
        services: &services,
        capabilities: &capabilities,
        problems: &problems,
        records: &records,
    })
}
