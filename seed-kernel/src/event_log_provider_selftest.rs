use crate::event_log::{
    Event, EventBindings, EventId, EventLog, ProviderBindingConsumption, ProviderBindingGateCheck,
    ProviderBindingGateSelfTestCase, ProviderContextHashes, ProviderContextInjectionAuthorization,
    ProviderContextInjectionGateCheck, ProviderContextInjectionGateSelfTestCase,
    ProviderExportAuditBinding, ProviderRequestBinding, ProviderRequestEnvelopeBinding,
    EVENT_CAPACITY, PROVIDER_BINDING_GATE_SELFTEST_CASES,
    PROVIDER_CONTEXT_INJECTION_GATE_SELFTEST_CASES,
};
use crate::event_log_evidence::{
    PROVIDER_BINDING_CONSUMPTION_EVIDENCE, PROVIDER_CONTEXT_INJECTION_AUTHORIZATION_EVIDENCE,
};

pub(crate) fn provider_context_binding_gate_selftest(
    context: ProviderContextHashes,
) -> [ProviderBindingGateSelfTestCase; PROVIDER_BINDING_GATE_SELFTEST_CASES] {
    [
        selftest_missing_export_audit_binding(context),
        selftest_denial_schema_substitution(context),
        selftest_stale_dropped_request_binding_event_id(context),
        selftest_stale_dropped_envelope_event_id(context),
        selftest_previous_boot_or_unretained_event_id(context),
        selftest_request_envelope_wrong_variant(context),
        selftest_positive_record_substitution(context),
        selftest_request_envelope_event_id_mismatch(context),
        selftest_request_id_mismatch(context),
        selftest_request_body_hash_mismatch(context),
        selftest_request_envelope_hash_mismatch(context),
        selftest_request_binding_hash_mismatch(context),
        selftest_provider_minimal_packet_hash_mismatch(context),
        selftest_exported_field_list_hash_mismatch(context),
        selftest_omitted_field_list_hash_mismatch(context),
        selftest_trust_bypass_record(context),
    ]
}

pub(crate) fn provider_context_injection_gate_selftest(
    context: ProviderContextHashes,
) -> [ProviderContextInjectionGateSelfTestCase; PROVIDER_CONTEXT_INJECTION_GATE_SELFTEST_CASES] {
    [
        selftest_missing_final_authorization(context),
        selftest_stale_dropped_final_authorization_event_id(context),
        selftest_final_authorization_schema_substitution(context),
        selftest_substituted_positive_final_authorization_record(context),
        selftest_final_authorization_body_hash_mismatch(context),
        selftest_final_authorization_trust_downgrade(context),
        selftest_body_attachment_without_final_authorization(context),
    ]
}

fn selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: ProviderBindingGateCheck,
) -> ProviderBindingGateSelfTestCase {
    ProviderBindingGateSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: check.status == expected_status && check.reason == expected_reason,
    }
}

fn injection_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: ProviderContextInjectionGateCheck,
) -> ProviderContextInjectionGateSelfTestCase {
    ProviderContextInjectionGateSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: check.status == expected_status && check.reason == expected_reason,
    }
}

fn selftest_missing_export_audit_binding(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let log = EventLog::new();
    selftest_case(
        "missing_export_audit_binding",
        "missing",
        "provider_context_export_audit_binding_missing",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_denial_schema_substitution(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let denial_event_id = record_selftest_request_denial(&mut log, context);
    let export = selftest_export_binding(1, envelope_event_id, denial_event_id, context);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "denial_schema_substitution",
        "rejected",
        "binding_denied_schema_or_wrong_variant",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_stale_dropped_request_binding_event_id(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    record_selftest_filler(&mut log, EVENT_CAPACITY);
    let export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "stale_dropped_request_binding_event_id",
        "rejected",
        "binding_stale_or_dropped_event_id",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_stale_dropped_envelope_event_id(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request = selftest_request_binding(1, envelope_event_id, context);
    let request_event_id = record_selftest_request_binding(&mut log, request);
    record_selftest_filler(&mut log, EVENT_CAPACITY - 2);
    let export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "stale_dropped_envelope_event_id",
        "rejected",
        "binding_stale_or_dropped_event_id",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_previous_boot_or_unretained_event_id(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let previous_boot_like_id = EventId { sequence: u64::MAX };
    let export = selftest_export_binding(1, envelope_event_id, previous_boot_like_id, context);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "previous_boot_or_unretained_event_id",
        "rejected",
        "binding_stale_or_dropped_event_id",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_request_envelope_wrong_variant(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let wrong_envelope_event_id = record_selftest_request_denial(&mut log, context);
    let request = selftest_request_binding(1, wrong_envelope_event_id, context);
    let request_event_id = record_selftest_request_binding(&mut log, request);
    let export = selftest_export_binding(1, wrong_envelope_event_id, request_event_id, context);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "request_envelope_wrong_variant",
        "rejected",
        "request_envelope_wrong_variant",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_positive_record_substitution(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let substituted_envelope_event_id = record_selftest_envelope(&mut log, 2);
    let mut substituted = selftest_request_binding(2, substituted_envelope_event_id, context);
    substituted.request_body_hash = tagged_hash(42);
    let substituted_event_id = record_selftest_request_binding(&mut log, substituted);
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.request_binding_event_id = substituted_event_id;
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "positive_record_substitution",
        "rejected",
        "binding_request_envelope_event_id_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_request_envelope_event_id_mismatch(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.request_envelope_event_id = EventId {
        sequence: envelope_event_id.sequence().saturating_add(99),
    };
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "request_envelope_event_id_mismatch",
        "rejected",
        "binding_request_envelope_event_id_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_request_id_mismatch(context: ProviderContextHashes) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let export = selftest_export_binding(2, envelope_event_id, request_event_id, context);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "request_id_mismatch",
        "rejected",
        "binding_request_id_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_request_body_hash_mismatch(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.request_body_hash = tagged_hash(43);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "request_body_hash_mismatch",
        "rejected",
        "binding_request_body_hash_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_request_envelope_hash_mismatch(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.request_envelope_hash = tagged_hash(44);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "request_envelope_hash_mismatch",
        "rejected",
        "binding_request_envelope_hash_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_request_binding_hash_mismatch(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.request_binding_hash = tagged_hash(45);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "request_binding_hash_mismatch",
        "rejected",
        "binding_request_binding_hash_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_provider_minimal_packet_hash_mismatch(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.context.projected_packet_hash = tagged_hash(46);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "provider_minimal_packet_hash_mismatch",
        "rejected",
        "binding_provider_minimal_packet_hash_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_exported_field_list_hash_mismatch(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.context.exported_field_list_hash = tagged_hash(47);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "exported_field_list_hash_mismatch",
        "rejected",
        "binding_exported_field_list_hash_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_omitted_field_list_hash_mismatch(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.context.omitted_field_list_hash = tagged_hash(48);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "omitted_field_list_hash_mismatch",
        "rejected",
        "binding_omitted_field_list_hash_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_trust_bypass_record(context: ProviderContextHashes) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let mut request = selftest_request_binding(1, envelope_event_id, context);
    request.development_tls_bypass = true;
    let request_event_id = record_selftest_request_binding(&mut log, request);
    let export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "trust_bypass_record",
        "rejected",
        "binding_trust_bypass_record",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_missing_final_authorization(
    context: ProviderContextHashes,
) -> ProviderContextInjectionGateSelfTestCase {
    let log = EventLog::new();
    injection_selftest_case(
        "missing_final_authorization",
        "missing",
        "final_injection_authorization_missing",
        log.check_provider_context_injection_gate(context, "pinned_spki_verified"),
    )
}

fn selftest_stale_dropped_final_authorization_event_id(
    context: ProviderContextHashes,
) -> ProviderContextInjectionGateSelfTestCase {
    let mut log = EventLog::new();
    let chain = record_selftest_injection_chain(&mut log, context);
    record_selftest_filler(&mut log, EVENT_CAPACITY);
    record_selftest_injection_authorization(
        &mut log,
        selftest_injection_authorization(chain, context),
    );

    injection_selftest_case(
        "stale_dropped_final_authorization_event_id",
        "rejected",
        "final_injection_authorization_stale_or_dropped_event_id",
        log.check_provider_context_injection_gate(context, "pinned_spki_verified"),
    )
}

fn selftest_final_authorization_schema_substitution(
    context: ProviderContextHashes,
) -> ProviderContextInjectionGateSelfTestCase {
    let mut log = EventLog::new();
    let chain = record_selftest_injection_chain(&mut log, context);
    let wrong_consumption_event_id = record_selftest_request_denial(&mut log, context);
    let mut authorization = selftest_injection_authorization(chain, context);
    authorization.binding_consumption_event_id = wrong_consumption_event_id;
    record_selftest_injection_authorization(&mut log, authorization);

    injection_selftest_case(
        "final_authorization_schema_substitution",
        "rejected",
        "final_injection_authorization_wrong_schema_or_variant",
        log.check_provider_context_injection_gate(context, "pinned_spki_verified"),
    )
}

fn selftest_substituted_positive_final_authorization_record(
    context: ProviderContextHashes,
) -> ProviderContextInjectionGateSelfTestCase {
    let mut log = EventLog::new();
    let chain = record_selftest_injection_chain(&mut log, context);
    let mut authorization = selftest_injection_authorization(chain, context);
    authorization.request_id = 2;
    record_selftest_injection_authorization(&mut log, authorization);

    injection_selftest_case(
        "substituted_positive_final_authorization_record",
        "rejected",
        "final_injection_authorization_substituted_record",
        log.check_provider_context_injection_gate(context, "pinned_spki_verified"),
    )
}

fn selftest_final_authorization_body_hash_mismatch(
    context: ProviderContextHashes,
) -> ProviderContextInjectionGateSelfTestCase {
    let mut log = EventLog::new();
    let chain = record_selftest_injection_chain(&mut log, context);
    let mut authorization = selftest_injection_authorization(chain, context);
    authorization.request_body_hash = tagged_hash(90);
    record_selftest_injection_authorization(&mut log, authorization);

    injection_selftest_case(
        "final_authorization_body_hash_mismatch",
        "rejected",
        "final_prewrite_body_hash_mismatch",
        log.check_provider_context_injection_gate(context, "pinned_spki_verified"),
    )
}

fn selftest_final_authorization_trust_downgrade(
    context: ProviderContextHashes,
) -> ProviderContextInjectionGateSelfTestCase {
    let mut log = EventLog::new();
    let chain = record_selftest_injection_chain(&mut log, context);
    record_selftest_injection_authorization(
        &mut log,
        selftest_injection_authorization(chain, context),
    );

    injection_selftest_case(
        "final_authorization_trust_downgrade",
        "rejected",
        "final_provider_trust_downgraded_before_write",
        log.check_provider_context_injection_gate(context, "pin_config_missing"),
    )
}

fn selftest_body_attachment_without_final_authorization(
    context: ProviderContextHashes,
) -> ProviderContextInjectionGateSelfTestCase {
    let mut log = EventLog::new();
    let chain = record_selftest_injection_chain(&mut log, context);
    let mut authorization = selftest_injection_authorization(chain, context);
    authorization.context_attached_to_provider_body = true;
    record_selftest_injection_authorization(&mut log, authorization);

    injection_selftest_case(
        "body_attachment_without_final_authorization",
        "rejected",
        "body_attachment_without_final_authorization",
        log.check_provider_context_injection_gate(context, "pinned_spki_verified"),
    )
}

#[derive(Clone, Copy)]
struct ProviderContextInjectionSelfTestChain {
    request_binding: ProviderRequestBinding,
    export_binding: ProviderExportAuditBinding,
    consumption: ProviderBindingConsumption,
    binding_consumption_event_id: EventId,
}

fn record_selftest_envelope(log: &mut EventLog, request_id: u32) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.provider_request.envelope_created",
        source_method: "provider.context_gate_selftest",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "synthetic_not_exported",
        requested_capability: "cap.provider.context_export.read",
        risk: "observe",
        subject: "selftest",
        resource: "current_boot.synthetic",
        reason: "synthetic_gate_input_not_global_evidence",
        evidence: &[],
        bindings: EventBindings::ProviderRequestEnvelope(ProviderRequestEnvelopeBinding {
            request_id,
            request_body_hash: tagged_hash(1),
            envelope_hash: tagged_hash(2),
            provider_trust_state: "pinned_spki_verified",
            provider_trust_positive: true,
            development_tls_bypass: false,
        }),
    })
}

fn record_selftest_request_binding(log: &mut EventLog, binding: ProviderRequestBinding) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.provider_context_export.request_binding_bound",
        source_method: "provider.context_gate_selftest",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "synthetic_not_exported",
        requested_capability: "cap.provider.context_export.read",
        risk: "observe",
        subject: "selftest",
        resource: "current_boot.synthetic",
        reason: "synthetic_gate_input_not_global_evidence",
        evidence: &[],
        bindings: EventBindings::ProviderRequestBound(binding),
    })
}

fn record_selftest_export_audit(
    log: &mut EventLog,
    binding: ProviderExportAuditBinding,
) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.provider_context_export.audit_binding_bound",
        source_method: "provider.context_gate_selftest",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "synthetic_not_exported",
        requested_capability: "cap.provider.context_export.read",
        risk: "observe",
        subject: "selftest",
        resource: "current_boot.synthetic",
        reason: "synthetic_gate_input_not_global_evidence",
        evidence: &[],
        bindings: EventBindings::ProviderExportAuditBound(binding),
    })
}

fn record_selftest_binding_consumption(
    log: &mut EventLog,
    binding: ProviderBindingConsumption,
) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.provider_context_export.binding_consumption_checked",
        source_method: "provider.context_injection_gate_selftest",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "synthetic_not_exported",
        requested_capability: "cap.provider.context_injection.read",
        risk: "observe",
        subject: "selftest",
        resource: "current_boot.synthetic",
        reason: "synthetic_gate_input_not_global_evidence",
        evidence: PROVIDER_BINDING_CONSUMPTION_EVIDENCE,
        bindings: EventBindings::ProviderBindingConsumption(binding),
    })
}

fn record_selftest_injection_authorization(
    log: &mut EventLog,
    binding: ProviderContextInjectionAuthorization,
) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.provider_context_injection.authorization_bound",
        source_method: "provider.context_injection_gate_selftest",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "synthetic_not_exported",
        requested_capability: "cap.provider.context_injection.read",
        risk: "observe",
        subject: "selftest",
        resource: "current_boot.synthetic",
        reason: "synthetic_gate_input_not_global_evidence",
        evidence: PROVIDER_CONTEXT_INJECTION_AUTHORIZATION_EVIDENCE,
        bindings: EventBindings::ProviderContextInjectionAuthorization(binding),
    })
}

fn record_selftest_request_denial(log: &mut EventLog, context: ProviderContextHashes) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.provider_context_export.request_binding_denied",
        source_method: "provider.context_gate_selftest",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "synthetic_not_exported",
        requested_capability: "cap.provider.context_export.read",
        risk: "observe",
        subject: "selftest",
        resource: "current_boot.synthetic",
        reason: "synthetic_denial_variant",
        evidence: &[],
        bindings: EventBindings::ProviderRequestBindingDenied(context),
    })
}

fn record_selftest_filler(log: &mut EventLog, count: usize) {
    let mut idx = 0usize;
    while idx < count {
        log.record(Event {
            sequence: 0,
            kind: "selftest.filler",
            source_method: "provider.context_gate_selftest",
            source_transport: "serial-console",
            classification: "local_only",
            outcome: "synthetic_not_exported",
            requested_capability: "cap.provider.context_export.read",
            risk: "observe",
            subject: "selftest",
            resource: "current_boot.synthetic",
            reason: "fills_ram_ring_to_exercise_retention",
            evidence: &[],
            bindings: EventBindings::None,
        });
        idx += 1;
    }
}

fn record_selftest_injection_chain(
    log: &mut EventLog,
    context: ProviderContextHashes,
) -> ProviderContextInjectionSelfTestChain {
    let request_envelope_event_id = record_selftest_envelope(log, 1);
    let request_binding = selftest_request_binding(1, request_envelope_event_id, context);
    let request_binding_event_id = record_selftest_request_binding(log, request_binding);
    let export_binding = selftest_export_binding(
        1,
        request_envelope_event_id,
        request_binding_event_id,
        context,
    );
    let export_audit_binding_event_id = record_selftest_export_audit(log, export_binding);
    let consumption = ProviderBindingConsumption {
        request_id: 1,
        request_envelope_event_id,
        request_binding_event_id,
        export_audit_binding_event_id,
        request_binding_hash: request_binding.request_binding_hash,
        export_audit_binding_hash: export_binding.export_audit_binding_hash,
        context,
    };
    let binding_consumption_event_id = record_selftest_binding_consumption(log, consumption);

    ProviderContextInjectionSelfTestChain {
        request_binding,
        export_binding,
        consumption,
        binding_consumption_event_id,
    }
}

fn selftest_request_binding(
    request_id: u32,
    request_envelope_event_id: EventId,
    context: ProviderContextHashes,
) -> ProviderRequestBinding {
    ProviderRequestBinding {
        request_id,
        request_envelope_event_id,
        request_body_hash: tagged_hash(1),
        request_envelope_hash: tagged_hash(2),
        request_binding_hash: tagged_hash(3),
        context,
        provider_trust_state: "pinned_spki_verified",
        development_tls_bypass: false,
    }
}

fn selftest_export_binding(
    request_id: u32,
    request_envelope_event_id: EventId,
    request_binding_event_id: EventId,
    context: ProviderContextHashes,
) -> ProviderExportAuditBinding {
    ProviderExportAuditBinding {
        request_id,
        request_envelope_event_id,
        request_binding_event_id,
        request_body_hash: tagged_hash(1),
        request_envelope_hash: tagged_hash(2),
        request_binding_hash: tagged_hash(3),
        export_audit_binding_hash: tagged_hash(4),
        context,
        provider_trust_state: "pinned_spki_verified",
        context_attached_to_provider_body: false,
    }
}

fn selftest_injection_authorization(
    chain: ProviderContextInjectionSelfTestChain,
    context: ProviderContextHashes,
) -> ProviderContextInjectionAuthorization {
    ProviderContextInjectionAuthorization {
        request_id: chain.consumption.request_id,
        request_envelope_event_id: chain.consumption.request_envelope_event_id,
        request_binding_event_id: chain.consumption.request_binding_event_id,
        export_audit_binding_event_id: chain.consumption.export_audit_binding_event_id,
        binding_consumption_event_id: chain.binding_consumption_event_id,
        request_body_hash: chain.request_binding.request_body_hash,
        request_envelope_hash: chain.request_binding.request_envelope_hash,
        request_binding_hash: chain.consumption.request_binding_hash,
        export_audit_binding_hash: chain.consumption.export_audit_binding_hash,
        context,
        provider_trust_state: chain.export_binding.provider_trust_state,
        final_authorization_hash: tagged_hash(5),
        context_attached_to_provider_body: false,
    }
}

fn tagged_hash(tag: u8) -> [u8; 32] {
    let mut hash = [tag; 32];
    hash[31] = tag.wrapping_mul(17);
    hash
}
