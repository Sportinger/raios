use crate::{
    agent_protocol_module_load_gate::emit_module_load_gate_event_binding,
    agent_protocol_provider::{
        emit_provider_context_hashes, emit_provider_minimal_projection,
        provider_context_block_reason,
    },
    agent_protocol_support::{
        begin_response, crlf, emit_inline_string_array, emit_record_value_fragment, end_response,
        indent, json_event_id, json_event_id_option, json_event_sequence, json_opt_str,
        json_sha256, json_sha256_option, json_str, method_eq, raw, raw_bool, raw_fmt, raw_line,
    },
    agent_protocol_system::{emit_problem_objects, emit_service_ids, emit_status_state},
    event_log, memory_store, provider, provider_trust, serial,
    system_status::SystemSnapshot,
    ui,
};

pub use raios_core::memory_context::{
    event_limit_arg, memory_context_estimated_tokens, memory_context_profile,
    memory_context_target_tokens, memory_method_arg, memory_mutation_method, MemoryContextPlan,
    MEMORY_MUTATION_METHODS,
};

pub(crate) fn emit_memory_profile() {
    begin_response("memory.profile");
    raw_line("      \"schema\": \"memory.profile.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"profiles\": [");
    raw_line("        {\"id\": \"diagnostic\", \"available\": true, \"target_tokens\": 4000, \"provider_export\": false, \"summary\": \"local current-boot facts for one diagnostic task\"},");
    raw_line("        {\"id\": \"planning\", \"available\": true, \"target_tokens\": 8000, \"provider_export\": false, \"summary\": \"local architecture and status handoff context\"},");
    raw_line("        {\"id\": \"provider_minimal\", \"available\": true, \"local_projection\": true, \"target_tokens\": 2000, \"provider_export\": false, \"blocked_by\": \"provider export requires positive provider trust plus current-boot export audit binding\", \"summary\": \"local read-only redaction projection for future provider context\"}");
    raw_line("      ],");
    raw_line("      \"read_methods\": [");
    raw_line("        \"memory.context\",");
    raw_line("        \"memory.query\",");
    raw_line("        \"memory.trace\",");
    raw_line("        \"memory.recent_events\",");
    raw_line("        \"audit.events\"");
    raw_line("      ],");
    raw_line("      \"mutation_policy\": \"denied_until_event_log_audit_policy_persistence_and_rollback_exist\"");
    end_response("memory.profile");
}

pub(crate) fn emit_memory_context(
    runtime: ui::RuntimeStatus,
    method: &str,
    event_id: event_log::EventId,
) {
    let status = SystemSnapshot::collect(None, runtime);
    let provider = provider::snapshot();
    let profile = memory_context_profile(method);

    begin_response("memory.context");
    raw_line("      \"schema\": \"raios.agent_context.v0\",");
    raw_line("      \"purpose\": \"current_boot_system_context\",");
    raw("      \"profile\": ");
    json_str(profile);
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"provider_export\": \"disabled\",");
    raw("      \"context_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("      \"audit_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw_line("      \"source_schemas\": [");
    raw_line("        \"system.snapshot.v0\",");
    raw_line("        \"system.capabilities.v0\",");
    raw_line("        \"service.inventory.v0\",");
    raw_line("        \"problem.list.v0\",");
    raw_line("        \"system.boot_log.v0\"");
    raw_line("      ],");
    raw("      \"budget\": {\"target_tokens\": ");
    raw_fmt(format_args!("{}", memory_context_target_tokens(profile)));
    raw(", \"estimated_tokens\": ");
    raw_fmt(format_args!("{}", memory_context_estimated_tokens(profile)));
    raw_line("},");
    raw_line("      \"authority_order\": [");
    raw_line("        \"current_snapshot\",");
    raw_line("        \"evidence\",");
    raw_line("        \"decision\",");
    raw_line("        \"service_state\",");
    raw_line("        \"summary\"");
    raw_line("      ],");
    raw_line("      \"included\": {");
    raw_line("        \"identity\": [\"mem.fact.identity.stage0\"],");
    raw_line("        \"policy\": [\"adr.0001\", \"adr.0004\"],");
    raw_line("        \"current\": [\"snapshot.current\", \"capabilities.current_boot\", \"service.inventory.current\", \"problem.list.current\"],");
    raw_line("        \"summaries\": [\"boot_log.summary.current\"]");
    raw_line("      },");
    raw_line("      \"current\": {");
    raw_line("        \"snapshot_id\": \"snapshot.current\",");
    raw_line("        \"status\": {");
    emit_status_state("framebuffer", status.framebuffer.state, true);
    emit_status_state("entropy", status.entropy.state, true);
    emit_status_state("usb_xhci", status.usb_xhci.state, true);
    emit_status_state("wifi", status.wifi.state, true);
    emit_status_state("network", status.network.state, true);
    emit_status_state("input", status.input.state, false);
    raw_line("        },");
    raw("        \"provider_trust_state\": ");
    json_str(provider.trust_state);
    raw_line(",");
    raw("        \"provider_api_key_state\": ");
    json_str(if provider.api_key_set {
        "set"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("        \"capability_posture\": \"observe_only_mutations_denied\",");
    raw_line("        \"services\": [");
    emit_service_ids(10);
    raw_line("        ],");
    raw_line("        \"problems\": [");
    emit_problem_objects(&status, &provider, 10);
    raw_line("        ]");
    raw_line("      },");
    if method_eq(profile, "provider_minimal") {
        raw_line("      \"provider_projection\": {");
        emit_provider_minimal_projection(&status, &provider, event_id);
        raw_line("      },");
    }
    raw_line("      \"records\": [");
    emit_memory_record(
        "mem.fact.identity.stage0",
        "fact",
        "current_snapshot",
        "public",
        "raiOS Stage-0 booted kernel identity",
        "system.describe",
        true,
    );
    emit_memory_record(
        "snapshot.current",
        "current_snapshot",
        "current_snapshot",
        "local_only",
        "current framebuffer, entropy, USB, Wi-Fi, network, input, provider, capabilities, and problems",
        "system.snapshot",
        true,
    );
    emit_memory_record(
        "capabilities.current_boot",
        "capability_index",
        "current_snapshot",
        "public",
        "observe-only capability posture and denied mutation vocabulary",
        "system.capabilities",
        true,
    );
    emit_memory_record(
        "service.inventory.current",
        "service_state",
        "service_state",
        "public",
        "static current-boot service inventory for the monolithic Stage-0 kernel",
        "service.inventory",
        true,
    );
    emit_memory_record(
        "problem.list.current",
        "problem_index",
        "current_snapshot",
        "public",
        "current known local problems and explicit gaps",
        "problem.list",
        true,
    );
    emit_memory_record(
        "boot_log.summary.current",
        "summary",
        "summary",
        "local_only",
        "serial boot log summary locator; raw lines remain local",
        "system.boot_log",
        true,
    );
    emit_memory_record(
        "adr.0001",
        "decision",
        "decision",
        "public",
        "build a raiOS-native agent protocol instead of porting the Codex CLI into Stage-0",
        "docs/architecture-decisions/0001-raios-agent-protocol.md",
        true,
    );
    emit_memory_record(
        "adr.0004",
        "decision",
        "decision",
        "public",
        "memory is typed local facts with bounded task-scoped agent_context packets, not prompt stuffing",
        "docs/architecture-decisions/0004-system-memory-and-agent-context.md",
        false,
    );
    let durable_context = memory_store::durable_memory_context();
    let durable_omitted = memory_store::durable_omitted_folds(&durable_context);
    raw_line("      ],");
    raw("      \"durable_records\": ");
    emit_record_value_fragment(memory_store::durable_records_array(&durable_context), 6);
    raw_line(",");
    raw_line("      \"omitted\": [");
    raw_line("        {\"kind\": \"raw_boot_log\", \"reason\": \"memory.context includes only a summary locator; use system.boot_log or memory.trace locally for raw lines\"},");
    raw_line("        {\"kind\": \"local_only_details\", \"reason\": \"details strings may contain IPs, PCI data, topology, request ids, or hashes\"},");
    raw_line("        {\"kind\": \"secret_values\", \"reason\": \"API keys, Wi-Fi passphrases, and raw secret values are never included\"},");
    raw_line("        {\"kind\": \"provider_export\", \"reason\": \"disabled until positive provider trust and current-boot provider export audit binding exist\"},");
    raw("        {\"kind\": \"provider_minimal\", \"reason\": ");
    json_str(provider_context_block_reason(provider.trust_state));
    raw("}");
    if durable_omitted.is_empty() {
        raw_line("");
    } else {
        raw_line(",");
        let mut omitted_idx = 0usize;
        while omitted_idx < durable_omitted.len() {
            raw("        ");
            emit_record_value_fragment(durable_omitted[omitted_idx].clone(), 8);
            if omitted_idx + 1 == durable_omitted.len() {
                raw_line("");
            } else {
                raw_line(",");
            }
            omitted_idx += 1;
        }
    }
    raw_line("      ]");
    end_response("memory.context");
}

pub(crate) fn emit_memory_query() {
    begin_response("memory.query");
    raw_line("      \"schema\": \"memory.query.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"query\": \"static_current_boot_index\",");
    raw_line("      \"records\": [");
    emit_memory_candidate(
        "mem.fact.identity.stage0",
        "fact",
        "public",
        "raiOS Stage-0 identity",
        true,
    );
    emit_memory_candidate(
        "snapshot.current",
        "current_snapshot",
        "local_only",
        "current typed system snapshot",
        true,
    );
    emit_memory_candidate(
        "snapshot.current.provider_minimal",
        "redacted_projection",
        "public",
        "provider_minimal projection of current status and provider trust",
        true,
    );
    emit_memory_candidate(
        "capabilities.current_boot",
        "capability_index",
        "public",
        "observe-only capability posture",
        true,
    );
    emit_memory_candidate(
        "service.inventory.current",
        "service_state",
        "public",
        "static service inventory",
        true,
    );
    emit_memory_candidate(
        "problem.list.current",
        "problem_index",
        "public",
        "current problem ids and severities",
        true,
    );
    emit_memory_candidate(
        "boot_log.summary.current",
        "summary",
        "local_only",
        "serial boot log summary locator",
        true,
    );
    emit_memory_candidate(
        "adr.0001",
        "decision",
        "public",
        "raiOS Agent Protocol instead of Codex CLI",
        true,
    );
    emit_memory_candidate(
        "adr.0004",
        "decision",
        "public",
        "System memory and agent context selection",
        false,
    );
    raw_line("      ],");
    raw_line("      \"semantic_index\": \"not_implemented_locator_only\"");
    end_response("memory.query");
}

pub(crate) fn emit_memory_trace(method: &str) {
    let id = memory_method_arg(method, "memory.trace");

    begin_response("memory.trace");
    raw_line("      \"schema\": \"memory.trace.v0\",");
    raw("      \"requested_id\": ");
    if id.is_empty() {
        raw("null");
    } else {
        json_str(id);
    }
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"records\": [");
    if id.is_empty() {
        emit_trace_record(
            "snapshot.current",
            "system.snapshot",
            "seed-kernel/src/agent_protocol.rs",
            true,
        );
        emit_trace_record(
            "snapshot.current.provider_minimal",
            "memory.context provider_minimal",
            "seed-kernel/src/agent_protocol.rs",
            true,
        );
        emit_trace_record(
            "service.inventory.current",
            "service.inventory",
            "seed-kernel/src/service_inventory.rs",
            true,
        );
        emit_trace_record(
            "problem.list.current",
            "problem.list",
            "seed-kernel/src/agent_protocol.rs",
            true,
        );
        emit_trace_record(
            "adr.0001",
            "decision",
            "docs/architecture-decisions/0001-raios-agent-protocol.md",
            true,
        );
        emit_trace_record(
            "adr.0004",
            "decision",
            "docs/architecture-decisions/0004-system-memory-and-agent-context.md",
            false,
        );
    } else {
        emit_single_trace_record(id);
    }
    raw_line("      ]");
    end_response("memory.trace");
}

pub(crate) fn emit_recent_events(method: &str) {
    let limit = event_limit_arg(method);
    let events = event_log::recent_events(limit);

    begin_response("memory.recent_events");
    raw_line("      \"schema\": \"event.log.v0\",");
    raw_line("      \"record_schema\": \"audit.event.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"retention\": \"ram_ring\",");
    raw_line("      \"persistence\": \"none\",");
    raw_line("      \"provider_export\": \"disabled\",");
    raw_line("      \"bounded\": true,");
    raw("      \"limit\": ");
    raw_fmt(format_args!("{}", events.limit));
    raw_line(",");
    raw("      \"capacity\": ");
    raw_fmt(format_args!("{}", events.capacity));
    raw_line(",");
    raw("      \"event_count\": ");
    raw_fmt(format_args!("{}", events.total_count));
    raw_line(",");
    raw("      \"returned\": ");
    raw_fmt(format_args!("{}", events.len));
    raw_line(",");
    raw("      \"dropped_before_sequence\": ");
    raw_fmt(format_args!("{}", events.dropped_before_sequence));
    raw_line(",");
    raw_line("      \"events\": [");

    let mut idx = 0usize;
    while idx < events.len {
        if let Some(event) = event_log::recent_event(events, idx) {
            emit_event(&event, idx + 1 != events.len);
        }
        idx += 1;
    }

    raw_line("      ]");
    end_response("memory.recent_events");
}

pub(crate) fn emit_memory_capability_denied(method: &'static str, event_id: event_log::EventId) {
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
    raw("    \"message\": ");
    json_str("memory mutation is denied until durable audit, policy, source retention, persistence, and rollback evidence exist");
    raw_line(",");
    raw_line("    \"required\": [");
    raw_line("      \"raios.audit_record.v0\",");
    raw_line("      \"policy_ledger\",");
    raw_line("      \"source_retention\",");
    raw_line("      \"redaction_transaction\",");
    raw_line("      \"raios.memory_persistence.v0\",");
    raw_line("      \"rollback_plan\"");
    raw_line("    ]");
    raw_line("  }");
    raw_line("}");
    serial::write_raw_fmt(format_args!("RAIOS_AGENT_END {}\r\n", method));
}

fn emit_event(event: &event_log::Event, comma: bool) {
    indent(8);
    raw("{\"schema\": \"audit.event.v0\", \"id\": ");
    json_event_sequence(event.sequence);
    raw(", \"scope\": \"current_boot\", \"sequence\": ");
    raw_fmt(format_args!("{}", event.sequence));
    raw(", \"kind\": ");
    json_str(event.kind);
    raw(", \"source_method\": ");
    json_str(event.source_method);
    raw(", \"source_transport\": ");
    json_str(event.source_transport);
    raw(", \"classification\": ");
    json_str(event.classification);
    raw(", \"outcome\": ");
    json_str(event.outcome);
    raw(", \"requested_capability\": ");
    json_str(event.requested_capability);
    raw(", \"risk\": ");
    json_str(event.risk);
    raw(", \"subject\": ");
    json_str(event.subject);
    raw(", \"resource\": ");
    json_str(event.resource);
    raw(", \"reason\": ");
    json_str(event.reason);
    raw(", \"evidence\": [");
    emit_inline_string_array(event.evidence);
    raw("], \"created_at\": {\"clock\": \"sequence_only\", \"millis\": null}");
    emit_event_bindings(event.kind, &event.bindings);
    raw(", \"persistence\": \"none\"}");
    if comma {
        raw(",");
    }
    crlf();
}

#[derive(Clone, Copy)]
enum BindingValue {
    Str(&'static str),
    OptStr(Option<&'static str>),
    Bool(bool),
    U64(u64),
    OptU64(Option<u64>),
    Sha256([u8; 32]),
    OptSha256(Option<[u8; 32]>),
    OptEventId(Option<event_log::EventId>),
}

#[derive(Clone, Copy)]
pub(crate) struct BindingField<F: Copy> {
    pub(crate) key: &'static str,
    pub(crate) field: F,
}

fn emit_binding_object<B, F: Copy>(
    binding: &B,
    kind: &str,
    fields: &[BindingField<F>],
    value: fn(&B, &str, F) -> BindingValue,
) {
    raw(", \"bindings\": {");
    let mut idx = 0usize;
    while idx < fields.len() {
        if idx != 0 {
            raw(", ");
        }
        emit_record_value_fragment(raios_core::record::Value::Str(fields[idx].key), 0);
        raw(": ");
        emit_binding_value(value(binding, kind, fields[idx].field));
        idx += 1;
    }
    raw("}");
}

pub(crate) fn emit_binding_object_direct<B, F: Copy>(
    binding: &B,
    kind: &str,
    fields: &[BindingField<F>],
    emit_value: fn(&B, &str, F),
) {
    raw(", \"bindings\": {");
    let mut idx = 0usize;
    while idx < fields.len() {
        if idx != 0 {
            raw(", ");
        }
        emit_record_value_fragment(raios_core::record::Value::Str(fields[idx].key), 0);
        raw(": ");
        emit_value(binding, kind, fields[idx].field);
        idx += 1;
    }
    raw("}");
}

macro_rules! define_direct_binding_fields {
    (
        $field_enum:ident,
        $fields_const:ident,
        $emit_value:ident,
        $binding_ty:ty,
        $binding:ident,
        $kind:ident;
        $(($variant:ident, $key:literal, $body:block),)+
    ) => {
        #[derive(Clone, Copy)]
        enum $field_enum {
            $($variant),+
        }

        const $fields_const: &[BindingField<$field_enum>] = &[
            $(BindingField {
                key: $key,
                field: $field_enum::$variant,
            }),+
        ];

        fn $emit_value($binding: &$binding_ty, $kind: &str, field: $field_enum) {
            match field {
                $($field_enum::$variant => $body),+
            }
        }
    };
}

pub(crate) use define_direct_binding_fields;

fn emit_binding_value(value: BindingValue) {
    match value {
        BindingValue::Str(value) => {
            emit_record_value_fragment(raios_core::record::Value::Str(value), 0)
        }
        BindingValue::OptStr(Some(value)) => {
            emit_record_value_fragment(raios_core::record::Value::Str(value), 0)
        }
        BindingValue::OptStr(None) => {
            emit_record_value_fragment(raios_core::record::Value::Null, 0)
        }
        BindingValue::Bool(value) => {
            emit_record_value_fragment(raios_core::record::Value::Bool(value), 0)
        }
        BindingValue::U64(value) => {
            emit_record_value_fragment(raios_core::record::Value::U64(value), 0)
        }
        BindingValue::OptU64(Some(value)) => {
            emit_record_value_fragment(raios_core::record::Value::U64(value), 0)
        }
        BindingValue::OptU64(None) => {
            emit_record_value_fragment(raios_core::record::Value::Null, 0)
        }
        BindingValue::Sha256(value) => {
            emit_record_value_fragment(raios_core::record::Value::Sha256(value), 0)
        }
        BindingValue::OptSha256(Some(value)) => {
            emit_record_value_fragment(raios_core::record::Value::Sha256(value), 0)
        }
        BindingValue::OptSha256(None) => {
            emit_record_value_fragment(raios_core::record::Value::Null, 0)
        }
        BindingValue::OptEventId(Some(value)) => emit_record_value_fragment(
            raios_core::record::Value::EventSequence(value.sequence()),
            0,
        ),
        BindingValue::OptEventId(None) => {
            emit_record_value_fragment(raios_core::record::Value::Null, 0)
        }
    }
}

macro_rules! define_hello_lifecycle_binding_fields {
    ($binding:ident, $kind:ident; $(($variant:ident, $key:literal, $value:expr,)),+ $(,)?) => {
        #[derive(Clone, Copy)]
        enum HelloLifecycleBindingField {
            $($variant),+
        }

        const HELLO_LIFECYCLE_BINDING_FIELDS: &[BindingField<HelloLifecycleBindingField>] = &[
            $(BindingField {
                key: $key,
                field: HelloLifecycleBindingField::$variant,
            }),+
        ];

        fn hello_lifecycle_binding_value(
            $binding: &event_log::HelloServiceLifecycleBinding,
            $kind: &str,
            field: HelloLifecycleBindingField,
        ) -> BindingValue {
            match field {
                $(HelloLifecycleBindingField::$variant => $value),+
            }
        }
    };
}

define_hello_lifecycle_binding_fields! { binding, kind;
    (Schema, "schema", BindingValue::Str(hello_lifecycle_binding_schema(kind, binding)),),
    (Status, "status", BindingValue::Str(hello_lifecycle_binding_status(kind, binding)),),
    (Scope, "scope", BindingValue::Str("current_boot"),),
    (Classification, "classification", BindingValue::Str("local_only"),),
    (LoadDescriptorSchema, "load_descriptor_schema", BindingValue::Str(binding.descriptor_schema),),
    (LoadDescriptorId, "load_descriptor_id", BindingValue::Str(binding.descriptor_id),),
    (LoadDescriptorSourceLocator, "load_descriptor_source_locator", BindingValue::Str(binding.descriptor_source_locator),),
    (LoadDescriptorSourceKind, "load_descriptor_source_kind", BindingValue::Str(binding.descriptor_source_kind),),
    (LoadDescriptorSourceHash, "load_descriptor_source_hash", BindingValue::Sha256(binding.descriptor_source_hash),),
    (LoadDescriptorSourceEnvelopeId, "load_descriptor_source_envelope_id", BindingValue::OptStr(binding.descriptor_source_envelope_id),),
    (LoadDescriptorSourceEnvelopeHash, "load_descriptor_source_envelope_hash", BindingValue::OptSha256(binding.descriptor_source_envelope_hash),),
    (LoadDescriptorSourceEnvelopePayloadHash, "load_descriptor_source_envelope_payload_hash", BindingValue::OptSha256(binding.descriptor_source_envelope_payload_hash),),
    (LoadDescriptorSourceEnvelopeTrustScope, "load_descriptor_source_envelope_trust_scope", BindingValue::OptStr(binding.descriptor_source_envelope_trust_scope),),
    (LoadDescriptorSourceSignatureAlgorithm, "load_descriptor_source_signature_algorithm", BindingValue::OptStr(binding.descriptor_source_signature_algorithm),),
    (LoadDescriptorSourceSignaturePublicKeyHash, "load_descriptor_source_signature_public_key_hash", BindingValue::OptSha256(binding.descriptor_source_signature_public_key_hash),),
    (LoadDescriptorSourceSignatureHash, "load_descriptor_source_signature_hash", BindingValue::OptSha256(binding.descriptor_source_signature_hash),),
    (LoadDescriptorSourceSignatureVerified, "load_descriptor_source_signature_verified", BindingValue::Bool(binding.descriptor_source_signature_verified),),
    (ArtifactIdentityId, "artifact_identity_id", BindingValue::Str(binding.artifact_identity_id),),
    (ArtifactIdentityHash, "artifact_identity_hash", BindingValue::Sha256(binding.artifact_identity_hash),),
    (ArtifactIdentityEnvelopeId, "artifact_identity_envelope_id", BindingValue::Str(binding.artifact_identity_envelope_id),),
    (ArtifactIdentityEnvelopeHash, "artifact_identity_envelope_hash", BindingValue::Sha256(binding.artifact_identity_envelope_hash),),
    (ArtifactIdentityEnvelopePayloadHash, "artifact_identity_envelope_payload_hash", BindingValue::Sha256(binding.artifact_identity_envelope_payload_hash),),
    (ArtifactIdentityEnvelopeTrustScope, "artifact_identity_envelope_trust_scope", BindingValue::Str(binding.artifact_identity_envelope_trust_scope),),
    (ArtifactIdentitySignatureAlgorithm, "artifact_identity_signature_algorithm", BindingValue::Str(binding.artifact_identity_signature_algorithm),),
    (ArtifactIdentitySignaturePublicKeyHash, "artifact_identity_signature_public_key_hash", BindingValue::Sha256(binding.artifact_identity_signature_public_key_hash),),
    (ArtifactIdentitySignatureHash, "artifact_identity_signature_hash", BindingValue::Sha256(binding.artifact_identity_signature_hash),),
    (ArtifactIdentitySignatureVerified, "artifact_identity_signature_verified", BindingValue::Bool(binding.artifact_identity_signature_verified),),
    (ArtifactIdentityValidated, "artifact_identity_validated", BindingValue::Bool(binding.artifact_identity_validated),),
    (ArtifactContentBindingId, "artifact_content_binding_id", BindingValue::Str(binding.artifact_content_binding_id),),
    (ArtifactContentBindingHash, "artifact_content_binding_hash", BindingValue::Sha256(binding.artifact_content_binding_hash),),
    (ArtifactContentSourceLocator, "artifact_content_source_locator", BindingValue::Str(binding.artifact_content_source_locator),),
    (ArtifactContentSourceHash, "artifact_content_source_hash", BindingValue::Sha256(binding.artifact_content_source_hash),),
    (ArtifactContentTrustEnvelopeId, "artifact_content_trust_envelope_id", BindingValue::Str(binding.artifact_content_trust_envelope_id),),
    (ArtifactContentTrustEnvelopeHash, "artifact_content_trust_envelope_hash", BindingValue::Sha256(binding.artifact_content_trust_envelope_hash),),
    (ArtifactContentTrustSignatureVerified, "artifact_content_trust_signature_verified", BindingValue::Bool(binding.artifact_content_trust_signature_verified),),
    (ArtifactContentValidated, "artifact_content_validated", BindingValue::Bool(binding.artifact_content_validated),),
    (ArtifactReferenceId, "artifact_reference_id", BindingValue::Str(binding.artifact_reference_id),),
    (ArtifactReferenceHash, "artifact_reference_hash", BindingValue::Sha256(binding.artifact_reference_hash),),
    (ArtifactBytesSha256, "artifact_bytes_sha256", BindingValue::Sha256(binding.artifact_reference_bytes_hash),),
    (ArtifactReferenceContentBindingHash, "artifact_reference_content_binding_hash", BindingValue::Sha256(binding.artifact_reference_content_binding_hash),),
    (ArtifactReferenceTrustEnvelopeId, "artifact_reference_trust_envelope_id", BindingValue::Str(binding.artifact_reference_trust_envelope_id),),
    (ArtifactReferenceTrustEnvelopeHash, "artifact_reference_trust_envelope_hash", BindingValue::Sha256(binding.artifact_reference_trust_envelope_hash),),
    (ArtifactReferenceTrustSignatureVerified, "artifact_reference_trust_signature_verified", BindingValue::Bool(binding.artifact_reference_trust_signature_verified),),
    (ArtifactReferenceValidated, "artifact_reference_validated", BindingValue::Bool(binding.artifact_reference_validated),),
    (ArtifactLoadPlanPreflightId, "artifact_load_plan_preflight_id", BindingValue::Str(binding.artifact_load_plan_preflight_id),),
    (ArtifactLoadPlanPreflightHash, "artifact_load_plan_preflight_hash", BindingValue::Sha256(binding.artifact_load_plan_preflight_hash),),
    (ArtifactLoadPlanPreflightStatus, "artifact_load_plan_preflight_status", BindingValue::Str(binding.artifact_load_plan_preflight_status),),
    (ArtifactLoadPlanPreflightAccepted, "artifact_load_plan_preflight_accepted", BindingValue::Bool(binding.artifact_load_plan_preflight_accepted),),
    (ServiceSlotIntentId, "service_slot_intent_id", BindingValue::Str(binding.service_slot_intent_id),),
    (RamOnlyServiceSlotId, "ram_only_service_slot_id", BindingValue::Str(binding.ram_only_service_slot_id),),
    (ServiceSlotActivationId, "service_slot_activation_id", BindingValue::Str(binding.service_slot_activation_id),),
    (ServiceSlotActivationHash, "service_slot_activation_hash", BindingValue::Sha256(binding.service_slot_activation_hash),),
    (ServiceSlotActivationStatus, "service_slot_activation_status", BindingValue::Str(binding.service_slot_activation_status),),
    (ServiceSlotActivationActive, "service_slot_activation_active", BindingValue::Bool(binding.service_slot_activation_active),),
    (HelloStateSchema, "hello_state_schema", BindingValue::Str(binding.hello_state_schema),),
    (HelloStateId, "hello_state_id", BindingValue::Str(binding.hello_state_id),),
    (HelloStateHash, "hello_state_hash", BindingValue::Sha256(binding.hello_state_hash),),
    (HelloStateCounter, "hello_state_counter", BindingValue::U64(binding.hello_state_counter),),
    (StateMigrationSchema, "state_migration_schema", BindingValue::OptStr(binding.state_migration_schema),),
    (StateMigrationId, "state_migration_id", BindingValue::OptStr(binding.state_migration_id),),
    (StateMigrationHash, "state_migration_hash", BindingValue::OptSha256(binding.state_migration_hash),),
    (MigrationFromVersion, "migration_from_version", BindingValue::OptStr(binding.migration_from_version),),
    (MigrationToVersion, "migration_to_version", BindingValue::OptStr(binding.migration_to_version),),
    (PreMigrationStateHash, "pre_migration_state_hash", BindingValue::OptSha256(binding.pre_migration_state_hash),),
    (PostMigrationStateHash, "post_migration_state_hash", BindingValue::OptSha256(binding.post_migration_state_hash),),
    (PreMigrationStateCounter, "pre_migration_state_counter", BindingValue::OptU64(binding.pre_migration_state_counter),),
    (PostMigrationStateCounter, "post_migration_state_counter", BindingValue::OptU64(binding.post_migration_state_counter),),
    (StateMigrationPreserved, "state_migration_preserved", BindingValue::Bool(binding.state_migration_preserved),),
    (StateMigrationAccepted, "state_migration_accepted", BindingValue::Bool(binding.state_migration_accepted),),
    (HotSwapProbationSchema, "hot_swap_probation_schema", BindingValue::OptStr(binding.hot_swap_probation_schema),),
    (HotSwapProbationId, "hot_swap_probation_id", BindingValue::OptStr(binding.hot_swap_probation_id),),
    (HotSwapProbationHash, "hot_swap_probation_hash", BindingValue::OptSha256(binding.hot_swap_probation_hash),),
    (HotSwapProbationStatus, "hot_swap_probation_status", BindingValue::OptStr(binding.hot_swap_probation_status),),
    (HotSwapProbationPreviousVersion, "hot_swap_probation_previous_version", BindingValue::OptStr(binding.hot_swap_probation_previous_version),),
    (HotSwapProbationNewVersion, "hot_swap_probation_new_version", BindingValue::OptStr(binding.hot_swap_probation_new_version),),
    (HotSwapProbationPreviousDescriptorSourceHash, "hot_swap_probation_previous_descriptor_source_hash", BindingValue::OptSha256(binding.hot_swap_probation_previous_descriptor_source_hash),),
    (HotSwapProbationNewDescriptorSourceHash, "hot_swap_probation_new_descriptor_source_hash", BindingValue::OptSha256(binding.hot_swap_probation_new_descriptor_source_hash),),
    (HotSwapProbationPreviousArtifactIdentityHash, "hot_swap_probation_previous_artifact_identity_hash", BindingValue::OptSha256(binding.hot_swap_probation_previous_artifact_identity_hash),),
    (HotSwapProbationNewArtifactIdentityHash, "hot_swap_probation_new_artifact_identity_hash", BindingValue::OptSha256(binding.hot_swap_probation_new_artifact_identity_hash),),
    (HotSwapProbationPreviousGeneration, "hot_swap_probation_previous_generation", BindingValue::OptU64(binding.hot_swap_probation_previous_generation),),
    (HotSwapProbationNewGeneration, "hot_swap_probation_new_generation", BindingValue::OptU64(binding.hot_swap_probation_new_generation),),
    (HotSwapProbationPreviousStateHash, "hot_swap_probation_previous_state_hash", BindingValue::OptSha256(binding.hot_swap_probation_previous_state_hash),),
    (HotSwapProbationNewStateHash, "hot_swap_probation_new_state_hash", BindingValue::OptSha256(binding.hot_swap_probation_new_state_hash),),
    (HotSwapProbationPreviousStateCounter, "hot_swap_probation_previous_state_counter", BindingValue::OptU64(binding.hot_swap_probation_previous_state_counter),),
    (HotSwapProbationNewStateCounter, "hot_swap_probation_new_state_counter", BindingValue::OptU64(binding.hot_swap_probation_new_state_counter),),
    (HotSwapProbationStateMigrationHash, "hot_swap_probation_state_migration_hash", BindingValue::OptSha256(binding.hot_swap_probation_state_migration_hash),),
    (HotSwapProbationAccepted, "hot_swap_probation_accepted", BindingValue::Bool(binding.hot_swap_probation_accepted),),
    (HotSwapProbationWritesPersistentState, "hot_swap_probation_writes_persistent_state", BindingValue::Bool(binding.hot_swap_probation_writes_persistent_state),),
    (HotSwapProbationWritesDurableAuditLog, "hot_swap_probation_writes_durable_audit_log", BindingValue::Bool(binding.hot_swap_probation_writes_durable_audit_log),),
    (HotSwapProbationInstallsRollbackPlan, "hot_swap_probation_installs_rollback_plan", BindingValue::Bool(binding.hot_swap_probation_installs_rollback_plan),),
    (HotSwapProbationAppliesRollback, "hot_swap_probation_applies_rollback", BindingValue::Bool(binding.hot_swap_probation_applies_rollback),),
    (RollbackPreviewSchema, "rollback_preview_schema", BindingValue::OptStr(binding.rollback_preview_schema),),
    (RollbackPreviewId, "rollback_preview_id", BindingValue::OptStr(binding.rollback_preview_id),),
    (RollbackPreviewHash, "rollback_preview_hash", BindingValue::OptSha256(binding.rollback_preview_hash),),
    (RollbackPreviewStatus, "rollback_preview_status", BindingValue::OptStr(binding.rollback_preview_status),),
    (RollbackApplySchema, "rollback_apply_schema", BindingValue::OptStr(binding.rollback_apply_schema),),
    (RollbackApplyId, "rollback_apply_id", BindingValue::OptStr(binding.rollback_apply_id),),
    (RollbackApplyHash, "rollback_apply_hash", BindingValue::OptSha256(binding.rollback_apply_hash),),
    (RollbackApplySourceDurablePolicyWriteAuthorityDecisionHash, "rollback_apply_source_durable_policy_write_authority_decision_hash", BindingValue::OptSha256(binding.rollback_apply_source_durable_policy_write_authority_decision_hash),),
    (RollbackApplySourceRecoveryRollbackInspectSourceReferenceHash, "rollback_apply_source_recovery_rollback_inspect_source_reference_hash", BindingValue::OptSha256(binding.rollback_apply_source_recovery_rollback_inspect_source_reference_hash),),
    (RollbackApplyStatus, "rollback_apply_status", BindingValue::OptStr(binding.rollback_apply_status),),
    (RollbackApplySourceDurablePolicyWriteAuthorityDecisionVerified, "rollback_apply_source_durable_policy_write_authority_decision_verified", BindingValue::Bool(binding.rollback_apply_source_durable_policy_write_authority_decision_verified),),
    (RollbackApplySourceRecoveryRollbackInspectSourceReferenceValidated, "rollback_apply_source_recovery_rollback_inspect_source_reference_validated", BindingValue::Bool(binding.rollback_apply_source_recovery_rollback_inspect_source_reference_validated),),
    (RollbackApplyAuthorized, "rollback_apply_authorized", BindingValue::Bool(binding.rollback_apply_authorized),),
    (RollbackApplyMutatesServiceState, "rollback_apply_mutates_service_state", BindingValue::Bool(binding.rollback_apply_mutates_service_state),),
    (RollbackTransactionPreflightSchema, "rollback_transaction_preflight_schema", BindingValue::OptStr(binding.rollback_transaction_preflight_schema),),
    (RollbackTransactionPreflightId, "rollback_transaction_preflight_id", BindingValue::OptStr(binding.rollback_transaction_preflight_id),),
    (RollbackTransactionPreflightHash, "rollback_transaction_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_preflight_hash),),
    (RollbackTransactionPreflightStatus, "rollback_transaction_preflight_status", BindingValue::OptStr(binding.rollback_transaction_preflight_status),),
    (RollbackTransactionAuthorityMissing, "rollback_transaction_authority_missing", BindingValue::Bool(binding.rollback_transaction_authority_missing),),
    (RollbackDurableAuditWriteAuthorityMissing, "rollback_durable_audit_write_authority_missing", BindingValue::Bool(binding.rollback_durable_audit_write_authority_missing),),
    (RollbackPersistentInstallAuthorityMissing, "rollback_persistent_install_authority_missing", BindingValue::Bool(binding.rollback_persistent_install_authority_missing),),
    (RollbackTransactionWritesDurableAuditLog, "rollback_transaction_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writes_durable_audit_log),),
    (RollbackTransactionWritesRollbackStore, "rollback_transaction_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writes_rollback_store),),
    (RollbackTransactionInstallsRollbackPlan, "rollback_transaction_installs_rollback_plan", BindingValue::Bool(binding.rollback_transaction_installs_rollback_plan),),
    (RollbackTransactionAppliesRollback, "rollback_transaction_applies_rollback", BindingValue::Bool(binding.rollback_transaction_applies_rollback),),
    (RollbackWriteAuthorityGateSchema, "rollback_write_authority_gate_schema", BindingValue::OptStr(binding.rollback_write_authority_gate_schema),),
    (RollbackWriteAuthorityGateId, "rollback_write_authority_gate_id", BindingValue::OptStr(binding.rollback_write_authority_gate_id),),
    (RollbackWriteAuthorityGateHash, "rollback_write_authority_gate_hash", BindingValue::OptSha256(binding.rollback_write_authority_gate_hash),),
    (RollbackWriteAuthorityGateStatus, "rollback_write_authority_gate_status", BindingValue::OptStr(binding.rollback_write_authority_gate_status),),
    (RollbackWriteAuthorityRequiredAuditSchema, "rollback_write_authority_required_audit_schema", BindingValue::OptStr(binding.rollback_write_authority_required_audit_schema),),
    (RollbackWriteAuthorityRequiredRollbackSchema, "rollback_write_authority_required_rollback_schema", BindingValue::OptStr(binding.rollback_write_authority_required_rollback_schema),),
    (RollbackAppendIntentGateSchema, "rollback_append_intent_gate_schema", BindingValue::OptStr(binding.rollback_append_intent_gate_schema),),
    (RollbackAppendIntentGateId, "rollback_append_intent_gate_id", BindingValue::OptStr(binding.rollback_append_intent_gate_id),),
    (RollbackAppendIntentGateHash, "rollback_append_intent_gate_hash", BindingValue::OptSha256(binding.rollback_append_intent_gate_hash),),
    (RollbackAppendIntentGateStatus, "rollback_append_intent_gate_status", BindingValue::OptStr(binding.rollback_append_intent_gate_status),),
    (RollbackAppendIntentRequiredAuditSchema, "rollback_append_intent_required_audit_schema", BindingValue::OptStr(binding.rollback_append_intent_required_audit_schema),),
    (RollbackAppendIntentRequiredRollbackSchema, "rollback_append_intent_required_rollback_schema", BindingValue::OptStr(binding.rollback_append_intent_required_rollback_schema),),
    (RollbackAppendIntentAvailable, "rollback_append_intent_available", BindingValue::Bool(binding.rollback_append_intent_available),),
    (RollbackAppendDurableAuditStoreAvailable, "rollback_append_durable_audit_store_available", BindingValue::Bool(binding.rollback_append_durable_audit_store_available),),
    (RollbackAppendStoreAvailable, "rollback_append_store_available", BindingValue::Bool(binding.rollback_append_store_available),),
    (RollbackAppendTransactionAppendAvailable, "rollback_append_transaction_append_available", BindingValue::Bool(binding.rollback_append_transaction_append_available),),
    (RollbackPayloadEnvelopeGateSchema, "rollback_payload_envelope_gate_schema", BindingValue::OptStr(binding.rollback_payload_envelope_gate_schema),),
    (RollbackPayloadEnvelopeGateId, "rollback_payload_envelope_gate_id", BindingValue::OptStr(binding.rollback_payload_envelope_gate_id),),
    (RollbackPayloadEnvelopeGateHash, "rollback_payload_envelope_gate_hash", BindingValue::OptSha256(binding.rollback_payload_envelope_gate_hash),),
    (RollbackPayloadEnvelopeGateStatus, "rollback_payload_envelope_gate_status", BindingValue::OptStr(binding.rollback_payload_envelope_gate_status),),
    (RollbackPayloadEnvelopeRequiredAuditSchema, "rollback_payload_envelope_required_audit_schema", BindingValue::OptStr(binding.rollback_payload_envelope_required_audit_schema),),
    (RollbackPayloadEnvelopeRequiredRollbackSchema, "rollback_payload_envelope_required_rollback_schema", BindingValue::OptStr(binding.rollback_payload_envelope_required_rollback_schema),),
    (RollbackPayloadSchema, "rollback_payload_schema", BindingValue::OptStr(binding.rollback_payload_schema),),
    (RollbackPayloadId, "rollback_payload_id", BindingValue::OptStr(binding.rollback_payload_id),),
    (RollbackPayloadHash, "rollback_payload_hash", BindingValue::OptSha256(binding.rollback_payload_hash),),
    (RollbackPayloadStatus, "rollback_payload_status", BindingValue::OptStr(binding.rollback_payload_status),),
    (RollbackPayloadProvenanceHash, "rollback_payload_provenance_hash", BindingValue::OptSha256(binding.rollback_payload_provenance_hash),),
    (RollbackPayloadWriterAvailable, "rollback_payload_writer_available", BindingValue::Bool(binding.rollback_payload_writer_available),),
    (RollbackPayloadDurableAuditStoreAvailable, "rollback_payload_durable_audit_store_available", BindingValue::Bool(binding.rollback_payload_durable_audit_store_available),),
    (RollbackPayloadStoreAvailable, "rollback_payload_store_available", BindingValue::Bool(binding.rollback_payload_store_available),),
    (RollbackPayloadTransactionAppendAvailable, "rollback_payload_transaction_append_available", BindingValue::Bool(binding.rollback_payload_transaction_append_available),),
    (RollbackTransactionWriterStorageAuthorityGateSchema, "rollback_transaction_writer_storage_authority_gate_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_authority_gate_schema),),
    (RollbackTransactionWriterStorageAuthorityGateId, "rollback_transaction_writer_storage_authority_gate_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_authority_gate_id),),
    (RollbackTransactionWriterStorageAuthorityGateHash, "rollback_transaction_writer_storage_authority_gate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_authority_gate_hash),),
    (RollbackTransactionWriterStorageAuthorityGateStatus, "rollback_transaction_writer_storage_authority_gate_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_authority_gate_status),),
    (RollbackTransactionWriterStorageRequiredAuditSchema, "rollback_transaction_writer_storage_required_audit_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_required_audit_schema),),
    (RollbackTransactionWriterStorageRequiredRollbackSchema, "rollback_transaction_writer_storage_required_rollback_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_required_rollback_schema),),
    (RollbackTransactionWriterStorageFoundationSchema, "rollback_transaction_writer_storage_foundation_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_foundation_schema),),
    (RollbackTransactionWriterStorageFoundationOwner, "rollback_transaction_writer_storage_foundation_owner", BindingValue::OptStr(binding.rollback_transaction_writer_storage_foundation_owner),),
    (RollbackTransactionWriterStorageAuthorityId, "rollback_transaction_writer_storage_authority_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_authority_id),),
    (RollbackTransactionWriterStorageAuthorityOwner, "rollback_transaction_writer_storage_authority_owner", BindingValue::OptStr(binding.rollback_transaction_writer_storage_authority_owner),),
    (RollbackTransactionWriterStorageAuditTargetId, "rollback_transaction_writer_storage_audit_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_audit_target_id),),
    (RollbackTransactionWriterStorageAuditTargetSchema, "rollback_transaction_writer_storage_audit_target_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_audit_target_schema),),
    (RollbackTransactionWriterStorageAppendTargetId, "rollback_transaction_writer_storage_append_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_target_id),),
    (RollbackTransactionWriterStorageAppendTargetSchema, "rollback_transaction_writer_storage_append_target_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_target_schema),),
    (RollbackTransactionWriterStorageTransactionWriterOwner, "rollback_transaction_writer_storage_transaction_writer_owner", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_writer_owner),),
    (RollbackTransactionWriterStorageAppendTargetOwnerSchema, "rollback_transaction_writer_storage_append_target_owner_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_target_owner_schema),),
    (RollbackTransactionWriterStorageAppendTargetOwnerId, "rollback_transaction_writer_storage_append_target_owner_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_target_owner_id),),
    (RollbackTransactionWriterStorageAppendTargetOwnerStatus, "rollback_transaction_writer_storage_append_target_owner_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_target_owner_status),),
    (RollbackTransactionWriterStorageAppendTargetOwnerReason, "rollback_transaction_writer_storage_append_target_owner_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_target_owner_reason),),
    (RollbackTransactionWriterStorageTransactionWriterReadinessSchema, "rollback_transaction_writer_storage_transaction_writer_readiness_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_writer_readiness_schema),),
    (RollbackTransactionWriterStorageTransactionWriterReadinessId, "rollback_transaction_writer_storage_transaction_writer_readiness_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_writer_readiness_id),),
    (RollbackTransactionWriterStorageTransactionWriterStatus, "rollback_transaction_writer_storage_transaction_writer_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_writer_status),),
    (RollbackTransactionWriterStorageTransactionWriterReason, "rollback_transaction_writer_storage_transaction_writer_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_writer_reason),),
    (RollbackTransactionWriterStorageBlockWritePathGateSchema, "rollback_transaction_writer_storage_block_write_path_gate_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_block_write_path_gate_schema),),
    (RollbackTransactionWriterStorageBlockWritePathGateId, "rollback_transaction_writer_storage_block_write_path_gate_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_block_write_path_gate_id),),
    (RollbackTransactionWriterStorageBlockWritePathGateStatus, "rollback_transaction_writer_storage_block_write_path_gate_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_block_write_path_gate_status),),
    (RollbackTransactionWriterStorageBlockWritePathGateReason, "rollback_transaction_writer_storage_block_write_path_gate_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_block_write_path_gate_reason),),
    (RollbackTransactionWriterStorageBlockWritePathAvailable, "rollback_transaction_writer_storage_block_write_path_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_block_write_path_available),),
    (RollbackTransactionWriterStorageReadOnlyBlockDriverId, "rollback_transaction_writer_storage_read_only_block_driver_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_read_only_block_driver_id),),
    (RollbackTransactionWriterStorageReadOnlyBlockDriverAvailable, "rollback_transaction_writer_storage_read_only_block_driver_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_read_only_block_driver_available),),
    (RollbackTransactionWriterStoragePartitionInventoryAvailable, "rollback_transaction_writer_storage_partition_inventory_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_partition_inventory_available),),
    (RollbackTransactionWriterStoragePartitionInventoryScheme, "rollback_transaction_writer_storage_partition_inventory_scheme", BindingValue::OptStr(binding.rollback_transaction_writer_storage_partition_inventory_scheme),),
    (RollbackTransactionWriterStorageScratchDryRunSchema, "rollback_transaction_writer_storage_scratch_dry_run_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_scratch_dry_run_schema),),
    (RollbackTransactionWriterStorageScratchDryRunId, "rollback_transaction_writer_storage_scratch_dry_run_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_scratch_dry_run_id),),
    (RollbackTransactionWriterStorageScratchDryRunStatus, "rollback_transaction_writer_storage_scratch_dry_run_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_scratch_dry_run_status),),
    (RollbackTransactionWriterStorageScratchDryRunReason, "rollback_transaction_writer_storage_scratch_dry_run_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_scratch_dry_run_reason),),
    (RollbackTransactionWriterStorageScratchWriteAuthorityId, "rollback_transaction_writer_storage_scratch_write_authority_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_scratch_write_authority_id),),
    (RollbackTransactionWriterStorageScratchRegionId, "rollback_transaction_writer_storage_scratch_region_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_scratch_region_id),),
    (RollbackTransactionWriterStorageScratchTargetStartLba, "rollback_transaction_writer_storage_scratch_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_scratch_target_start_lba),),
    (RollbackTransactionWriterStorageScratchTargetLbaCount, "rollback_transaction_writer_storage_scratch_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_scratch_target_lba_count),),
    (RollbackTransactionWriterStorageScratchTargetByteCount, "rollback_transaction_writer_storage_scratch_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_scratch_target_byte_count),),
    (RollbackTransactionWriterStorageScratchTargetOwned, "rollback_transaction_writer_storage_scratch_target_owned", BindingValue::Bool(binding.rollback_transaction_writer_storage_scratch_target_owned),),
    (RollbackTransactionWriterStorageScratchTargetWithinBounds, "rollback_transaction_writer_storage_scratch_target_within_bounds", BindingValue::Bool(binding.rollback_transaction_writer_storage_scratch_target_within_bounds),),
    (RollbackTransactionWriterStorageScratchTargetNoMetadataOverlap, "rollback_transaction_writer_storage_scratch_target_no_metadata_overlap", BindingValue::Bool(binding.rollback_transaction_writer_storage_scratch_target_no_metadata_overlap),),
    (RollbackTransactionWriterStorageScratchTargetReady, "rollback_transaction_writer_storage_scratch_target_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_scratch_target_ready),),
    (RollbackTransactionWriterStorageAppendRecordDryRunSchema, "rollback_transaction_writer_storage_append_record_dry_run_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_record_dry_run_schema),),
    (RollbackTransactionWriterStorageAppendRecordDryRunId, "rollback_transaction_writer_storage_append_record_dry_run_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_record_dry_run_id),),
    (RollbackTransactionWriterStorageAppendRecordDryRunStatus, "rollback_transaction_writer_storage_append_record_dry_run_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_record_dry_run_status),),
    (RollbackTransactionWriterStorageAppendRecordDryRunReason, "rollback_transaction_writer_storage_append_record_dry_run_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_record_dry_run_reason),),
    (RollbackTransactionWriterStorageAppendRecordDryRunHash, "rollback_transaction_writer_storage_append_record_dry_run_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_record_dry_run_hash),),
    (RollbackTransactionWriterStorageAppendRecordCanonicalization, "rollback_transaction_writer_storage_append_record_canonicalization", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_record_canonicalization),),
    (RollbackTransactionWriterStorageAppendRecordAuditImageHash, "rollback_transaction_writer_storage_append_record_audit_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_record_audit_image_hash),),
    (RollbackTransactionWriterStorageAppendRecordAuditByteLength, "rollback_transaction_writer_storage_append_record_audit_byte_length", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_record_audit_byte_length),),
    (RollbackTransactionWriterStorageAppendRecordRollbackImageHash, "rollback_transaction_writer_storage_append_record_rollback_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_record_rollback_image_hash),),
    (RollbackTransactionWriterStorageAppendRecordRollbackByteLength, "rollback_transaction_writer_storage_append_record_rollback_byte_length", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_record_rollback_byte_length),),
    (RollbackTransactionWriterStorageAppendRecordTotalByteLength, "rollback_transaction_writer_storage_append_record_total_byte_length", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_record_total_byte_length),),
    (RollbackTransactionWriterStorageAppendRecordTargetStartLba, "rollback_transaction_writer_storage_append_record_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_record_target_start_lba),),
    (RollbackTransactionWriterStorageAppendRecordTargetLbaCount, "rollback_transaction_writer_storage_append_record_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_record_target_lba_count),),
    (RollbackTransactionWriterStorageAppendRecordTargetByteCount, "rollback_transaction_writer_storage_append_record_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_record_target_byte_count),),
    (RollbackTransactionWriterStorageAppendRecordSourcePayloadHash, "rollback_transaction_writer_storage_append_record_source_payload_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_record_source_payload_hash),),
    (RollbackTransactionWriterStorageAppendRecordSourceProvenanceHash, "rollback_transaction_writer_storage_append_record_source_provenance_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_record_source_provenance_hash),),
    (RollbackTransactionWriterStorageAppendRecordTargetRangeReady, "rollback_transaction_writer_storage_append_record_target_range_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_record_target_range_ready),),
    (RollbackTransactionWriterStorageAppendRecordAuthorizesAppend, "rollback_transaction_writer_storage_append_record_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_record_authorizes_append),),
    (RollbackTransactionWriterStorageAppendRecordWritesDurableAuditLog, "rollback_transaction_writer_storage_append_record_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_record_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageAppendRecordWritesRollbackStore, "rollback_transaction_writer_storage_append_record_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_record_writes_rollback_store),),
    (RollbackTransactionWriterStorageAppendRecordAppendsRollbackTransaction, "rollback_transaction_writer_storage_append_record_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_record_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageAppendRecordWriteAttempted, "rollback_transaction_writer_storage_append_record_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_record_write_attempted),),
    (RollbackTransactionWriterStorageAppendSectorPlanDryRunSchema, "rollback_transaction_writer_storage_append_sector_plan_dry_run_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_sector_plan_dry_run_schema),),
    (RollbackTransactionWriterStorageAppendSectorPlanDryRunId, "rollback_transaction_writer_storage_append_sector_plan_dry_run_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_sector_plan_dry_run_id),),
    (RollbackTransactionWriterStorageAppendSectorPlanDryRunStatus, "rollback_transaction_writer_storage_append_sector_plan_dry_run_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_sector_plan_dry_run_status),),
    (RollbackTransactionWriterStorageAppendSectorPlanDryRunReason, "rollback_transaction_writer_storage_append_sector_plan_dry_run_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_sector_plan_dry_run_reason),),
    (RollbackTransactionWriterStorageAppendSectorPlanHash, "rollback_transaction_writer_storage_append_sector_plan_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_sector_plan_hash),),
    (RollbackTransactionWriterStorageAppendSectorPlanCanonicalization, "rollback_transaction_writer_storage_append_sector_plan_canonicalization", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_sector_plan_canonicalization),),
    (RollbackTransactionWriterStorageAppendSectorImageHash, "rollback_transaction_writer_storage_append_sector_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_sector_image_hash),),
    (RollbackTransactionWriterStorageAppendSectorSizeBytes, "rollback_transaction_writer_storage_append_sector_size_bytes", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_sector_size_bytes),),
    (RollbackTransactionWriterStorageAppendSectorAuditRecordOffset, "rollback_transaction_writer_storage_append_sector_audit_record_offset", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_sector_audit_record_offset),),
    (RollbackTransactionWriterStorageAppendSectorAuditRecordByteLength, "rollback_transaction_writer_storage_append_sector_audit_record_byte_length", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_sector_audit_record_byte_length),),
    (RollbackTransactionWriterStorageAppendSectorRollbackTransactionOffset, "rollback_transaction_writer_storage_append_sector_rollback_transaction_offset", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_sector_rollback_transaction_offset),),
    (RollbackTransactionWriterStorageAppendSectorRollbackTransactionByteLength, "rollback_transaction_writer_storage_append_sector_rollback_transaction_byte_length", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_sector_rollback_transaction_byte_length),),
    (RollbackTransactionWriterStorageAppendSectorPaddingPolicy, "rollback_transaction_writer_storage_append_sector_padding_policy", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_sector_padding_policy),),
    (RollbackTransactionWriterStorageAppendSectorPaddingOffset, "rollback_transaction_writer_storage_append_sector_padding_offset", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_sector_padding_offset),),
    (RollbackTransactionWriterStorageAppendSectorPaddingByteLength, "rollback_transaction_writer_storage_append_sector_padding_byte_length", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_sector_padding_byte_length),),
    (RollbackTransactionWriterStorageAppendSectorTargetStartLba, "rollback_transaction_writer_storage_append_sector_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_sector_target_start_lba),),
    (RollbackTransactionWriterStorageAppendSectorTargetLbaCount, "rollback_transaction_writer_storage_append_sector_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_sector_target_lba_count),),
    (RollbackTransactionWriterStorageAppendSectorTargetByteCount, "rollback_transaction_writer_storage_append_sector_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_sector_target_byte_count),),
    (RollbackTransactionWriterStorageAppendSectorSourceRecordHash, "rollback_transaction_writer_storage_append_sector_source_record_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_sector_source_record_hash),),
    (RollbackTransactionWriterStorageAppendSectorTargetRangeReady, "rollback_transaction_writer_storage_append_sector_target_range_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_target_range_ready),),
    (RollbackTransactionWriterStorageAppendSectorAuthorizesAppend, "rollback_transaction_writer_storage_append_sector_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_authorizes_append),),
    (RollbackTransactionWriterStorageAppendSectorWritesDurableAuditLog, "rollback_transaction_writer_storage_append_sector_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageAppendSectorWritesRollbackStore, "rollback_transaction_writer_storage_append_sector_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_writes_rollback_store),),
    (RollbackTransactionWriterStorageAppendSectorAppendsRollbackTransaction, "rollback_transaction_writer_storage_append_sector_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageAppendSectorWriteAttempted, "rollback_transaction_writer_storage_append_sector_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_write_attempted),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackDryRunSchema, "rollback_transaction_writer_storage_append_sector_write_readback_dry_run_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_sector_write_readback_dry_run_schema),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackDryRunId, "rollback_transaction_writer_storage_append_sector_write_readback_dry_run_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_sector_write_readback_dry_run_id),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackDryRunStatus, "rollback_transaction_writer_storage_append_sector_write_readback_dry_run_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_sector_write_readback_dry_run_status),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackDryRunReason, "rollback_transaction_writer_storage_append_sector_write_readback_dry_run_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_sector_write_readback_dry_run_reason),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackDryRunHash, "rollback_transaction_writer_storage_append_sector_write_readback_dry_run_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_sector_write_readback_dry_run_hash),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackSourcePlanHash, "rollback_transaction_writer_storage_append_sector_write_readback_source_plan_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_sector_write_readback_source_plan_hash),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackPlannedImageHash, "rollback_transaction_writer_storage_append_sector_write_readback_planned_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_sector_write_readback_planned_image_hash),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackReadbackImageHash, "rollback_transaction_writer_storage_append_sector_write_readback_readback_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_sector_write_readback_readback_image_hash),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackTargetStartLba, "rollback_transaction_writer_storage_append_sector_write_readback_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_sector_write_readback_target_start_lba),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackTargetLbaCount, "rollback_transaction_writer_storage_append_sector_write_readback_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_sector_write_readback_target_lba_count),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackTargetByteCount, "rollback_transaction_writer_storage_append_sector_write_readback_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_sector_write_readback_target_byte_count),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackLabelFound, "rollback_transaction_writer_storage_append_sector_write_readback_label_found", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_write_readback_label_found),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackTargetRangeReady, "rollback_transaction_writer_storage_append_sector_write_readback_target_range_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_write_readback_target_range_ready),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackWriteAttempted, "rollback_transaction_writer_storage_append_sector_write_readback_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_write_readback_write_attempted),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackWriteCompleted, "rollback_transaction_writer_storage_append_sector_write_readback_write_completed", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_write_readback_write_completed),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackReadbackCompleted, "rollback_transaction_writer_storage_append_sector_write_readback_readback_completed", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_write_readback_readback_completed),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackMatchesPlannedImage, "rollback_transaction_writer_storage_append_sector_write_readback_matches_planned_image", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_write_readback_matches_planned_image),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackAuthorizesAppend, "rollback_transaction_writer_storage_append_sector_write_readback_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_write_readback_authorizes_append),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackWritesDurableAuditLog, "rollback_transaction_writer_storage_append_sector_write_readback_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_write_readback_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackWritesRollbackStore, "rollback_transaction_writer_storage_append_sector_write_readback_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_write_readback_writes_rollback_store),),
    (RollbackTransactionWriterStorageAppendSectorWriteReadbackAppendsRollbackTransaction, "rollback_transaction_writer_storage_append_sector_write_readback_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_sector_write_readback_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightSchema, "rollback_transaction_writer_storage_durable_append_authority_preflight_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_schema),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightId, "rollback_transaction_writer_storage_durable_append_authority_preflight_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_id),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightStatus, "rollback_transaction_writer_storage_durable_append_authority_preflight_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_status),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightReason, "rollback_transaction_writer_storage_durable_append_authority_preflight_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_reason),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightHash, "rollback_transaction_writer_storage_durable_append_authority_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightSourceWriteReadbackHash, "rollback_transaction_writer_storage_durable_append_authority_preflight_source_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_source_write_readback_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_durable_append_authority_preflight_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_append_authority_preflight_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightRemainingDenialReason, "rollback_transaction_writer_storage_durable_append_authority_preflight_remaining_denial_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_remaining_denial_reason),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightSchema, "rollback_transaction_writer_storage_durable_writer_policy_preflight_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_schema),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightId, "rollback_transaction_writer_storage_durable_writer_policy_preflight_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_id),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightStatus, "rollback_transaction_writer_storage_durable_writer_policy_preflight_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_status),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightReason, "rollback_transaction_writer_storage_durable_writer_policy_preflight_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_reason),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightHash, "rollback_transaction_writer_storage_durable_writer_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightSourceAppendRecordHash, "rollback_transaction_writer_storage_durable_writer_policy_preflight_source_append_record_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_source_append_record_hash),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightSourceSectorPlanHash, "rollback_transaction_writer_storage_durable_writer_policy_preflight_source_sector_plan_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_source_sector_plan_hash),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_durable_writer_policy_preflight_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightAuditLedgerTargetId, "rollback_transaction_writer_storage_durable_writer_policy_preflight_audit_ledger_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_audit_ledger_target_id),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightAuditRecordSchema, "rollback_transaction_writer_storage_durable_writer_policy_preflight_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_audit_record_schema),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightRollbackStoreTargetId, "rollback_transaction_writer_storage_durable_writer_policy_preflight_rollback_store_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_rollback_store_target_id),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightRollbackTransactionSchema, "rollback_transaction_writer_storage_durable_writer_policy_preflight_rollback_transaction_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_rollback_transaction_schema),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightTargetStartLba, "rollback_transaction_writer_storage_durable_writer_policy_preflight_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_target_start_lba),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightTargetLbaCount, "rollback_transaction_writer_storage_durable_writer_policy_preflight_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_target_lba_count),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightTargetByteCount, "rollback_transaction_writer_storage_durable_writer_policy_preflight_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_target_byte_count),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightTargetRangeReady, "rollback_transaction_writer_storage_durable_writer_policy_preflight_target_range_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_target_range_ready),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_writer_policy_preflight_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightDurableAuditWriterAvailable, "rollback_transaction_writer_storage_durable_writer_policy_preflight_durable_audit_writer_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_durable_audit_writer_available),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightRollbackStoreWriterAvailable, "rollback_transaction_writer_storage_durable_writer_policy_preflight_rollback_store_writer_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_rollback_store_writer_available),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightTransactionAppendWriterAvailable, "rollback_transaction_writer_storage_durable_writer_policy_preflight_transaction_append_writer_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_transaction_append_writer_available),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightAuthorizesAppend, "rollback_transaction_writer_storage_durable_writer_policy_preflight_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_authorizes_append),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_writer_policy_preflight_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightWritesRollbackStore, "rollback_transaction_writer_storage_durable_writer_policy_preflight_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_writer_policy_preflight_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurableWriterPolicyPreflightWriteAttempted, "rollback_transaction_writer_storage_durable_writer_policy_preflight_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_writer_policy_preflight_write_attempted),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateSchema, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_schema),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateId, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_id),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateStatus, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_status),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateReason, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_reason),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateHash, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_hash),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateSourceWriterPolicyPreflightHash, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_source_writer_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_source_writer_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateSourceAppendRecordHash, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_source_append_record_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_source_append_record_hash),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateSourceSectorPlanHash, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_source_sector_plan_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_source_sector_plan_hash),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateAuditLedgerTargetId, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_audit_ledger_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_audit_ledger_target_id),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateAuditRecordSchema, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_audit_record_schema),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateRollbackStoreTargetId, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_rollback_store_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_rollback_store_target_id),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateRollbackTransactionSchema, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_rollback_transaction_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_rollback_transaction_schema),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateTargetStartLba, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_target_start_lba),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateTargetLbaCount, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_target_lba_count),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateTargetByteCount, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_target_byte_count),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateTargetRangeReady, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_target_range_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_target_range_ready),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateAppendEngineAvailable, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_append_engine_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_append_engine_available),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateDurableAuditWriterAvailable, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_durable_audit_writer_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_durable_audit_writer_available),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateRollbackStoreWriterAvailable, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_rollback_store_writer_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_rollback_store_writer_available),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateTransactionAppendWriterAvailable, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_transaction_append_writer_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_transaction_append_writer_available),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateAuthorizesAppend, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_authorizes_append),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateAuthorizesTransactionAppend, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_authorizes_transaction_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_authorizes_transaction_append),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateWritesRollbackStore, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurableAppendTransactionAuthorizationGateWriteAttempted, "rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_transaction_authorization_gate_write_attempted),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionSchema, "rollback_transaction_writer_storage_append_engine_readiness_decision_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_schema),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionId, "rollback_transaction_writer_storage_append_engine_readiness_decision_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_id),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionStatus, "rollback_transaction_writer_storage_append_engine_readiness_decision_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_status),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionReason, "rollback_transaction_writer_storage_append_engine_readiness_decision_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_reason),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionHash, "rollback_transaction_writer_storage_append_engine_readiness_decision_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_hash),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionSourceAuthorizationGateHash, "rollback_transaction_writer_storage_append_engine_readiness_decision_source_authorization_gate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_source_authorization_gate_hash),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionSourceWriterPolicyPreflightHash, "rollback_transaction_writer_storage_append_engine_readiness_decision_source_writer_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_source_writer_policy_preflight_hash),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionSourceAppendRecordHash, "rollback_transaction_writer_storage_append_engine_readiness_decision_source_append_record_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_source_append_record_hash),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionSourceSectorPlanHash, "rollback_transaction_writer_storage_append_engine_readiness_decision_source_sector_plan_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_source_sector_plan_hash),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_append_engine_readiness_decision_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionTargetStartLba, "rollback_transaction_writer_storage_append_engine_readiness_decision_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_target_start_lba),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionTargetLbaCount, "rollback_transaction_writer_storage_append_engine_readiness_decision_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_target_lba_count),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionTargetByteCount, "rollback_transaction_writer_storage_append_engine_readiness_decision_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_target_byte_count),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionTargetRangeReady, "rollback_transaction_writer_storage_append_engine_readiness_decision_target_range_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_target_range_ready),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_append_engine_readiness_decision_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionAppendEngineAvailable, "rollback_transaction_writer_storage_append_engine_readiness_decision_append_engine_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_append_engine_available),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionDurableAuditWriterAvailable, "rollback_transaction_writer_storage_append_engine_readiness_decision_durable_audit_writer_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_durable_audit_writer_available),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionRollbackStoreWriterAvailable, "rollback_transaction_writer_storage_append_engine_readiness_decision_rollback_store_writer_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_rollback_store_writer_available),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionTransactionAppendWriterAvailable, "rollback_transaction_writer_storage_append_engine_readiness_decision_transaction_append_writer_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_transaction_append_writer_available),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionReady, "rollback_transaction_writer_storage_append_engine_readiness_decision_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_ready),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionAuthorizesAppend, "rollback_transaction_writer_storage_append_engine_readiness_decision_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_authorizes_append),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionAuthorizesTransactionAppend, "rollback_transaction_writer_storage_append_engine_readiness_decision_authorizes_transaction_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_authorizes_transaction_append),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionWritesDurableAuditLog, "rollback_transaction_writer_storage_append_engine_readiness_decision_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionWritesRollbackStore, "rollback_transaction_writer_storage_append_engine_readiness_decision_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_writes_rollback_store),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionAppendsRollbackTransaction, "rollback_transaction_writer_storage_append_engine_readiness_decision_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageAppendEngineReadinessDecisionWriteAttempted, "rollback_transaction_writer_storage_append_engine_readiness_decision_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_engine_readiness_decision_write_attempted),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightSchema, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_schema),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightId, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_id),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightStatus, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_status),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightReason, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_reason),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightHash, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightSourceContractSchema,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_source_contract_schema",
        BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_source_contract_schema),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightSourceContractId,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_source_contract_id",
        BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_source_contract_id),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightSourceContractStatus,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_source_contract_status",
        BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_source_contract_status),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightSourceContractReason,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_source_contract_reason",
        BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_source_contract_reason),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightOwnerMethod,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_owner_method",
        BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_owner_method),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightAppendTargetOwnerId,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_append_target_owner_id",
        BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_append_target_owner_id),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightStorageAuthorityId,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_storage_authority_id",
        BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_storage_authority_id),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightAuditLedgerTargetId,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_audit_ledger_target_id",
        BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_audit_ledger_target_id),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightAuditRecordSchema,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_audit_record_schema",
        BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_audit_record_schema),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightRollbackStoreTargetId,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_rollback_store_target_id",
        BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_rollback_store_target_id),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightRollbackTransactionSchema,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_rollback_transaction_schema",
        BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_rollback_transaction_schema),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightTargetRegionStartLba,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_target_region_start_lba",
        BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_target_region_start_lba),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightTargetRegionLbaCount,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_target_region_lba_count",
        BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_target_region_lba_count),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightTargetByteCount,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_target_byte_count",
        BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_target_byte_count),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightSourceContractTargetRangeReady,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_source_contract_target_range_ready",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_source_contract_target_range_ready),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightOwnerIdsVerified,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_owner_ids_verified",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_owner_ids_verified),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightTargetIdsVerified,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_target_ids_verified",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_target_ids_verified),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightTargetSpanVerified,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_target_span_verified",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_target_span_verified),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightSchemaIdsVerified,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_schema_ids_verified",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_schema_ids_verified),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightMediaWriteAuthorityRequired,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_media_write_authority_required",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_media_write_authority_required),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightMediaWriteAuthorityAvailable,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_media_write_authority_available",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_media_write_authority_available),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightMediaWriteAuthorityReason,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_media_write_authority_reason",
        BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_media_write_authority_reason),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightDurableAuditPolicyRequired,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_durable_audit_policy_required",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_durable_audit_policy_required),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightDurableAuditPolicyAvailable,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_durable_audit_policy_available",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_durable_audit_policy_available),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightDurableAuditPolicyReason,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_durable_audit_policy_reason",
        BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_durable_audit_policy_reason),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightAuthorizesMediaWrite,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_authorizes_media_write",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_authorizes_media_write),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightAuthorizesAppend,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_authorizes_append",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_authorizes_append),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightWritesDurableAuditLog,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_writes_durable_audit_log",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_writes_durable_audit_log),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightWritesRollbackStore,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_writes_rollback_store",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_writes_rollback_store),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightAppendsRollbackTransaction,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_appends_rollback_transaction",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_appends_rollback_transaction),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMediaWritePolicyPreflightWriteAttempted,
        "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_write_attempted",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_media_write_policy_preflight_write_attempted),
        ),

    (RollbackTransactionWriterStorageMediaWriteAuthorityGateSchema, "rollback_transaction_writer_storage_media_write_authority_gate_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_media_write_authority_gate_schema),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateId, "rollback_transaction_writer_storage_media_write_authority_gate_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_media_write_authority_gate_id),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateStatus, "rollback_transaction_writer_storage_media_write_authority_gate_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_media_write_authority_gate_status),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateReason, "rollback_transaction_writer_storage_media_write_authority_gate_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_media_write_authority_gate_reason),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateHash, "rollback_transaction_writer_storage_media_write_authority_gate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_media_write_authority_gate_hash),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateSourceDurableAppendAuthorityPreflightHash, "rollback_transaction_writer_storage_media_write_authority_gate_source_durable_append_authority_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_media_write_authority_gate_source_durable_append_authority_preflight_hash),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateSourcePolicyPreflightHash, "rollback_transaction_writer_storage_media_write_authority_gate_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_media_write_authority_gate_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_media_write_authority_gate_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_media_write_authority_gate_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateSourceContractSchema, "rollback_transaction_writer_storage_media_write_authority_gate_source_contract_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_media_write_authority_gate_source_contract_schema),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateSourceContractId, "rollback_transaction_writer_storage_media_write_authority_gate_source_contract_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_media_write_authority_gate_source_contract_id),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateSourceContractStatus, "rollback_transaction_writer_storage_media_write_authority_gate_source_contract_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_media_write_authority_gate_source_contract_status),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateSourceContractReason, "rollback_transaction_writer_storage_media_write_authority_gate_source_contract_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_media_write_authority_gate_source_contract_reason),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateTargetRegionStartLba, "rollback_transaction_writer_storage_media_write_authority_gate_target_region_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_media_write_authority_gate_target_region_start_lba),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateTargetRegionLbaCount, "rollback_transaction_writer_storage_media_write_authority_gate_target_region_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_media_write_authority_gate_target_region_lba_count),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateTargetByteCount, "rollback_transaction_writer_storage_media_write_authority_gate_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_media_write_authority_gate_target_byte_count),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateSourceContractTargetRangeReady, "rollback_transaction_writer_storage_media_write_authority_gate_source_contract_target_range_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_source_contract_target_range_ready),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateOwnerIdsVerified, "rollback_transaction_writer_storage_media_write_authority_gate_owner_ids_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_owner_ids_verified),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateTargetIdsVerified, "rollback_transaction_writer_storage_media_write_authority_gate_target_ids_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_target_ids_verified),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateTargetSpanVerified, "rollback_transaction_writer_storage_media_write_authority_gate_target_span_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_target_span_verified),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateSchemaIdsVerified, "rollback_transaction_writer_storage_media_write_authority_gate_schema_ids_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_schema_ids_verified),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateMediaWriteAuthorityRequired, "rollback_transaction_writer_storage_media_write_authority_gate_media_write_authority_required", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_media_write_authority_required),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_media_write_authority_gate_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_media_write_authority_available),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateMediaWriteAuthorityReason, "rollback_transaction_writer_storage_media_write_authority_gate_media_write_authority_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_media_write_authority_gate_media_write_authority_reason),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_media_write_authority_gate_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateDurableAuditPolicyRequired, "rollback_transaction_writer_storage_media_write_authority_gate_durable_audit_policy_required", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_durable_audit_policy_required),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_media_write_authority_gate_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateDurableAuditPolicyReason, "rollback_transaction_writer_storage_media_write_authority_gate_durable_audit_policy_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_media_write_authority_gate_durable_audit_policy_reason),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateAuthorizesMediaWrite, "rollback_transaction_writer_storage_media_write_authority_gate_authorizes_media_write", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_authorizes_media_write),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateAuthorizesAppend, "rollback_transaction_writer_storage_media_write_authority_gate_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_authorizes_append),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateWritesDurableAuditLog, "rollback_transaction_writer_storage_media_write_authority_gate_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateWritesRollbackStore, "rollback_transaction_writer_storage_media_write_authority_gate_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_writes_rollback_store),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateAppendsRollbackTransaction, "rollback_transaction_writer_storage_media_write_authority_gate_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateTargetRegionWriteAttempted, "rollback_transaction_writer_storage_media_write_authority_gate_target_region_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_target_region_write_attempted),),
    (RollbackTransactionWriterStorageMediaWriteAuthorityGateWriteAttempted, "rollback_transaction_writer_storage_media_write_authority_gate_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_media_write_authority_gate_write_attempted),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionSchema, "rollback_transaction_writer_storage_durable_append_authority_decision_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_decision_schema),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionId, "rollback_transaction_writer_storage_durable_append_authority_decision_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_decision_id),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionStatus, "rollback_transaction_writer_storage_durable_append_authority_decision_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_decision_status),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionReason, "rollback_transaction_writer_storage_durable_append_authority_decision_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_decision_reason),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionHash, "rollback_transaction_writer_storage_durable_append_authority_decision_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_decision_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionSourceDurableAppendAuthorityPreflightHash, "rollback_transaction_writer_storage_durable_append_authority_decision_source_durable_append_authority_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_decision_source_durable_append_authority_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionSourceWriterPolicyPreflightHash, "rollback_transaction_writer_storage_durable_append_authority_decision_source_writer_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_decision_source_writer_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionSourceAppendEngineReadinessDecisionHash, "rollback_transaction_writer_storage_durable_append_authority_decision_source_append_engine_readiness_decision_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_decision_source_append_engine_readiness_decision_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionSourceMediaWriteAuthorityGateHash, "rollback_transaction_writer_storage_durable_append_authority_decision_source_media_write_authority_gate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_decision_source_media_write_authority_gate_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionSourcePolicyPreflightHash, "rollback_transaction_writer_storage_durable_append_authority_decision_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_decision_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_durable_append_authority_decision_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_decision_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionTargetStartLba, "rollback_transaction_writer_storage_durable_append_authority_decision_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_authority_decision_target_start_lba),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionTargetLbaCount, "rollback_transaction_writer_storage_durable_append_authority_decision_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_authority_decision_target_lba_count),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionTargetByteCount, "rollback_transaction_writer_storage_durable_append_authority_decision_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_authority_decision_target_byte_count),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionWriterPolicyReady, "rollback_transaction_writer_storage_durable_append_authority_decision_writer_policy_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_decision_writer_policy_ready),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionAppendEngineReady, "rollback_transaction_writer_storage_durable_append_authority_decision_append_engine_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_decision_append_engine_ready),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionMediaWriteGateReady, "rollback_transaction_writer_storage_durable_append_authority_decision_media_write_gate_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_decision_media_write_gate_ready),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_append_authority_decision_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_decision_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_durable_append_authority_decision_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_decision_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_durable_append_authority_decision_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_decision_durable_append_authority_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionAuthorizesAppend, "rollback_transaction_writer_storage_durable_append_authority_decision_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_decision_authorizes_append),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionAuthorizesTransactionAppend, "rollback_transaction_writer_storage_durable_append_authority_decision_authorizes_transaction_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_decision_authorizes_transaction_append),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_append_authority_decision_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_decision_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionWritesRollbackStore, "rollback_transaction_writer_storage_durable_append_authority_decision_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_decision_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_append_authority_decision_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_decision_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityDecisionWriteAttempted, "rollback_transaction_writer_storage_durable_append_authority_decision_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_decision_write_attempted),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionSchema, "rollback_transaction_writer_storage_durable_audit_policy_decision_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionId, "rollback_transaction_writer_storage_durable_audit_policy_decision_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_id),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionStatus, "rollback_transaction_writer_storage_durable_audit_policy_decision_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_status),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionReason, "rollback_transaction_writer_storage_durable_audit_policy_decision_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_reason),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionHash, "rollback_transaction_writer_storage_durable_audit_policy_decision_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionSourceDurableAppendAuthorityDecisionHash, "rollback_transaction_writer_storage_durable_audit_policy_decision_source_durable_append_authority_decision_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_source_durable_append_authority_decision_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionSourcePolicyPreflightHash, "rollback_transaction_writer_storage_durable_audit_policy_decision_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionSourceMediaWriteAuthorityGateHash, "rollback_transaction_writer_storage_durable_audit_policy_decision_source_media_write_authority_gate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_source_media_write_authority_gate_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_durable_audit_policy_decision_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionTargetStartLba, "rollback_transaction_writer_storage_durable_audit_policy_decision_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_target_start_lba),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionTargetLbaCount, "rollback_transaction_writer_storage_durable_audit_policy_decision_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_target_lba_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionTargetByteCount, "rollback_transaction_writer_storage_durable_audit_policy_decision_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_target_byte_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionAppendEngineReady, "rollback_transaction_writer_storage_durable_audit_policy_decision_append_engine_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_append_engine_ready),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionMediaWritePolicyVerified, "rollback_transaction_writer_storage_durable_audit_policy_decision_media_write_policy_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_media_write_policy_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_decision_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_decision_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_durable_append_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_durable_audit_policy_decision_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionAuthorizesAppend, "rollback_transaction_writer_storage_durable_audit_policy_decision_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_authorizes_append),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_audit_policy_decision_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionWritesRollbackStore, "rollback_transaction_writer_storage_durable_audit_policy_decision_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_audit_policy_decision_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurableAuditPolicyDecisionWriteAttempted, "rollback_transaction_writer_storage_durable_audit_policy_decision_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_decision_write_attempted),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateSchema, "rollback_transaction_writer_storage_durable_audit_policy_candidate_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateId, "rollback_transaction_writer_storage_durable_audit_policy_candidate_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_id),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateStatus, "rollback_transaction_writer_storage_durable_audit_policy_candidate_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_status),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateReason, "rollback_transaction_writer_storage_durable_audit_policy_candidate_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_reason),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateHash, "rollback_transaction_writer_storage_durable_audit_policy_candidate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateSourceDurableAuditPolicyDecisionHash, "rollback_transaction_writer_storage_durable_audit_policy_candidate_source_durable_audit_policy_decision_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_source_durable_audit_policy_decision_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateSourceAuditRecordImageHash, "rollback_transaction_writer_storage_durable_audit_policy_candidate_source_audit_record_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_source_audit_record_image_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateSourcePolicyPreflightHash, "rollback_transaction_writer_storage_durable_audit_policy_candidate_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_durable_audit_policy_candidate_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateAuditRecordSchema, "rollback_transaction_writer_storage_durable_audit_policy_candidate_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_audit_record_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateTargetStartLba, "rollback_transaction_writer_storage_durable_audit_policy_candidate_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_target_start_lba),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateTargetLbaCount, "rollback_transaction_writer_storage_durable_audit_policy_candidate_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_target_lba_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateTargetByteCount, "rollback_transaction_writer_storage_durable_audit_policy_candidate_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_target_byte_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateMediaWritePolicyVerified, "rollback_transaction_writer_storage_durable_audit_policy_candidate_media_write_policy_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_media_write_policy_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateAvailable, "rollback_transaction_writer_storage_durable_audit_policy_candidate_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_durable_audit_policy_candidate_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_candidate_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_durable_append_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateAuthorizesAppend, "rollback_transaction_writer_storage_durable_audit_policy_candidate_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_authorizes_append),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_audit_policy_candidate_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateWritesRollbackStore, "rollback_transaction_writer_storage_durable_audit_policy_candidate_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_audit_policy_candidate_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurableAuditPolicyCandidateWriteAttempted, "rollback_transaction_writer_storage_durable_audit_policy_candidate_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_candidate_write_attempted),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateSchema, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateId, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_id),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateStatus, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_status),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateReason, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_reason),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateHash, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateSourceCandidateHash, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_source_candidate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_source_candidate_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateSourceDecisionHash, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_source_decision_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_source_decision_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateSourceAuditRecordImageHash, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_source_audit_record_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_source_audit_record_image_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateSourcePolicyPreflightHash, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateAuditRecordSchema, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_audit_record_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateTargetStartLba, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_target_start_lba),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateTargetLbaCount, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_target_lba_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateTargetByteCount, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_target_byte_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateCandidateAvailable, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_candidate_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_candidate_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateMediaWritePolicyVerified, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_media_write_policy_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_media_write_policy_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateDurablePolicyLedgerAvailable, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_durable_policy_ledger_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_durable_policy_ledger_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_durable_append_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateAuthorizesAppend, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_authorizes_append),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateWritesRollbackStore, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAcceptanceGateWriteAttempted, "rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_acceptance_gate_write_attempted),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateSchema, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateId, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_id),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateStatus, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_status),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateReason, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_reason),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateHash, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateSourceAcceptanceGateHash, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_acceptance_gate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_acceptance_gate_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateSourceCandidateHash, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_candidate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_candidate_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateSourceDecisionHash, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_decision_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_decision_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateSourceAuditRecordImageHash, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_audit_record_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_audit_record_image_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateSourcePolicyPreflightHash, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateAuditRecordSchema, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_audit_record_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateTargetStartLba, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_target_start_lba),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateTargetLbaCount, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_target_lba_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateTargetByteCount, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_target_byte_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateReadOnlyAvailable, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_read_only_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_read_only_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateCandidateAvailable, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_candidate_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_candidate_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateMediaWritePolicyVerified, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_media_write_policy_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_media_write_policy_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateDurablePolicyLedgerAvailable, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_durable_policy_ledger_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_durable_policy_ledger_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_durable_append_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateAuthorizesAppend, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_authorizes_append),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateWritesRollbackStore, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerCandidateWriteAttempted, "rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_candidate_write_attempted),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultSchema, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultId, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_id),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultStatus, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_status),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultReason, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_reason),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultHash, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultSourceLedgerCandidateHash, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_ledger_candidate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_ledger_candidate_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultSourceAcceptanceGateHash, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_acceptance_gate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_acceptance_gate_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultSourceCandidateHash, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_candidate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_candidate_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultSourceDecisionHash, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_decision_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_decision_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultSourceAuditRecordImageHash, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_audit_record_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_audit_record_image_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultSourcePolicyPreflightHash, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultSourceTargetRegionWriteReadbackHash,
        "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_target_region_write_readback_hash",
        BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_source_target_region_write_readback_hash),
        ),

    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultAuditRecordSchema, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_audit_record_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultTargetStartLba, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_target_start_lba),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultTargetLbaCount, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_target_lba_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultTargetByteCount, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_target_byte_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultReadOnlyAvailable, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_read_only_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_read_only_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultLedgerEvidenceVerified, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_ledger_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_ledger_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultDurablePolicyLedgerAvailable, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_durable_policy_ledger_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_durable_policy_ledger_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_durable_append_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultAuthorizesAppend, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_authorizes_append),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultWritesRollbackStore, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurableAuditPolicyLedgerAwareAcceptanceResultWriteAttempted, "rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_ledger_aware_acceptance_result_write_attempted),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilitySchema, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityId, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_id),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityStatus, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_status),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityReason, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_reason),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityHash, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilitySourceResultHash, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_source_result_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_source_result_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilitySourceLedgerCandidateHash, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_source_ledger_candidate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_source_ledger_candidate_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilitySourcePolicyPreflightHash, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilitySourceTargetRegionWriteReadbackHash,
        "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_source_target_region_write_readback_hash",
        BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_source_target_region_write_readback_hash),
        ),

    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityAuditLedgerTargetId, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_audit_ledger_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_audit_ledger_target_id),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityAuditRecordSchema, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_audit_record_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityRollbackStoreTargetId, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_rollback_store_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_rollback_store_target_id),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityRollbackTransactionSchema, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_rollback_transaction_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_rollback_transaction_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityTargetStartLba, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_start_lba),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityTargetLbaCount, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_lba_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityTargetByteCount, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_byte_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityLedgerEvidenceVerified, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_ledger_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_ledger_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityMediaWritePolicyVerified, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_media_write_policy_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_media_write_policy_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityTargetRegionWriteReadbackVerified, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_region_write_readback_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_region_write_readback_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityTargetSpanVerified, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_span_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_target_span_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityAuditRollbackTargetIdsVerified, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_audit_rollback_target_ids_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_audit_rollback_target_ids_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityDurablePolicyLedgerAvailable, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_durable_policy_ledger_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_durable_policy_ledger_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_durable_append_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityAuthorizesMediaWrite, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_authorizes_media_write", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_authorizes_media_write),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityAuthorizesAppend, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_authorizes_append),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityWritesRollbackStore, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurableAuditPolicyWriteAuthorityAvailabilityWriteAttempted, "rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_write_authority_availability_write_attempted),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilitySchema, "rollback_transaction_writer_storage_durable_policy_ledger_availability_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_schema),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityId, "rollback_transaction_writer_storage_durable_policy_ledger_availability_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_id),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityStatus, "rollback_transaction_writer_storage_durable_policy_ledger_availability_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_status),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityReason, "rollback_transaction_writer_storage_durable_policy_ledger_availability_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_reason),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityHash, "rollback_transaction_writer_storage_durable_policy_ledger_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_hash),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilitySourceWriteAuthorityAvailabilityHash, "rollback_transaction_writer_storage_durable_policy_ledger_availability_source_write_authority_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_source_write_authority_availability_hash),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilitySourceResultHash, "rollback_transaction_writer_storage_durable_policy_ledger_availability_source_result_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_source_result_hash),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilitySourceLedgerCandidateHash, "rollback_transaction_writer_storage_durable_policy_ledger_availability_source_ledger_candidate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_source_ledger_candidate_hash),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilitySourcePolicyPreflightHash, "rollback_transaction_writer_storage_durable_policy_ledger_availability_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilitySourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_durable_policy_ledger_availability_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityAuditLedgerTargetId, "rollback_transaction_writer_storage_durable_policy_ledger_availability_audit_ledger_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_audit_ledger_target_id),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityAuditRecordSchema, "rollback_transaction_writer_storage_durable_policy_ledger_availability_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_audit_record_schema),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityRollbackStoreTargetId, "rollback_transaction_writer_storage_durable_policy_ledger_availability_rollback_store_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_rollback_store_target_id),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityRollbackTransactionSchema, "rollback_transaction_writer_storage_durable_policy_ledger_availability_rollback_transaction_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_rollback_transaction_schema),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityTargetStartLba, "rollback_transaction_writer_storage_durable_policy_ledger_availability_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_target_start_lba),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityTargetLbaCount, "rollback_transaction_writer_storage_durable_policy_ledger_availability_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_target_lba_count),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityTargetByteCount, "rollback_transaction_writer_storage_durable_policy_ledger_availability_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_target_byte_count),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityWriteAuthorityEvidenceVerified, "rollback_transaction_writer_storage_durable_policy_ledger_availability_write_authority_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_write_authority_evidence_verified),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityLedgerEvidenceVerified, "rollback_transaction_writer_storage_durable_policy_ledger_availability_ledger_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_ledger_evidence_verified),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityMediaWritePolicyVerified, "rollback_transaction_writer_storage_durable_policy_ledger_availability_media_write_policy_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_media_write_policy_verified),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityTargetRegionWriteReadbackVerified, "rollback_transaction_writer_storage_durable_policy_ledger_availability_target_region_write_readback_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_target_region_write_readback_verified),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityTargetSpanVerified, "rollback_transaction_writer_storage_durable_policy_ledger_availability_target_span_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_target_span_verified),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityAuditRollbackTargetIdsVerified, "rollback_transaction_writer_storage_durable_policy_ledger_availability_audit_rollback_target_ids_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_audit_rollback_target_ids_verified),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_policy_ledger_availability_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_policy_ledger_availability_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_write_authority_available),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDurablePolicyLedgerAvailable, "rollback_transaction_writer_storage_durable_policy_ledger_availability_durable_policy_ledger_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_durable_policy_ledger_available),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_durable_policy_ledger_availability_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_durable_policy_ledger_availability_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_durable_append_authority_available),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityAuthorizesMediaWrite, "rollback_transaction_writer_storage_durable_policy_ledger_availability_authorizes_media_write", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_authorizes_media_write),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityAuthorizesAppend, "rollback_transaction_writer_storage_durable_policy_ledger_availability_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_authorizes_append),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_policy_ledger_availability_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityWritesRollbackStore, "rollback_transaction_writer_storage_durable_policy_ledger_availability_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_policy_ledger_availability_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityWriteAttempted, "rollback_transaction_writer_storage_durable_policy_ledger_availability_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_write_attempted),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunSchema, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_schema),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunId, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_id),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunStatus, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_status),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunReason, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_reason),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunHash, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_hash),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunSourcePolicyLedgerAvailabilityHash, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_policy_ledger_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_policy_ledger_availability_hash),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunSourceWriteAuthorityAvailabilityHash, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_write_authority_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_write_authority_availability_hash),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunSourceResultHash, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_result_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_result_hash),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunSourceLedgerCandidateHash, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_ledger_candidate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_ledger_candidate_hash),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunSourcePolicyPreflightHash, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunSourceAuthorityDenialGateHash, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_authority_denial_gate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_authority_denial_gate_hash),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunSourceTransactionAppendAvailabilityDecisionHash,
        "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_transaction_append_availability_decision_hash",
        BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_source_transaction_append_availability_decision_hash),
        ),

    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunAuditLedgerTargetId, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_audit_ledger_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_audit_ledger_target_id),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunAuditRecordSchema, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_audit_record_schema),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunRollbackStoreTargetId, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_rollback_store_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_rollback_store_target_id),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunRollbackTransactionSchema, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_rollback_transaction_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_rollback_transaction_schema),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunTargetStartLba, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_target_start_lba),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunTargetLbaCount, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_target_lba_count),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunTargetByteCount, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_target_byte_count),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunPolicyLedgerAvailabilityEvidenceVerified, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_policy_ledger_availability_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_policy_ledger_availability_evidence_verified),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunWriteAuthorityEvidenceVerified, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_write_authority_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_write_authority_evidence_verified),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunLedgerEvidenceVerified, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_ledger_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_ledger_evidence_verified),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunMediaWritePolicyVerified, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_media_write_policy_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_media_write_policy_verified),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunTargetRegionWriteReadbackVerified, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_target_region_write_readback_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_target_region_write_readback_verified),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunTransactionAppendDenialGateVerified, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_transaction_append_denial_gate_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_transaction_append_denial_gate_verified),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunTargetSpanVerified, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_target_span_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_target_span_verified),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunAuditRollbackTargetIdsVerified, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_audit_rollback_target_ids_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_audit_rollback_target_ids_verified),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_write_authority_available),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunDurablePolicyLedgerAvailable, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_durable_policy_ledger_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_durable_policy_ledger_available),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_durable_append_authority_available),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunTransactionAppendAvailable, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_transaction_append_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_transaction_append_available),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunAuthorizesMediaWrite, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_authorizes_media_write", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_authorizes_media_write),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunAuthorizesAppend, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_authorizes_append),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunAuthorizesTransactionAppend, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_authorizes_transaction_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_authorizes_transaction_append),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunWritesRollbackStore, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunWriteAttempted, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_write_attempted),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunAppliesRollback, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_applies_rollback", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_applies_rollback),),
    (RollbackTransactionWriterStorageDurablePolicyLedgerAvailabilityDryRunInstallsRollbackState, "rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_installs_rollback_state", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_ledger_availability_dry_run_installs_rollback_state),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilitySchema, "rollback_transaction_writer_storage_durable_audit_policy_availability_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityId, "rollback_transaction_writer_storage_durable_audit_policy_availability_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_id),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityStatus, "rollback_transaction_writer_storage_durable_audit_policy_availability_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_status),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityReason, "rollback_transaction_writer_storage_durable_audit_policy_availability_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_reason),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilitySourcePolicyLedgerAvailabilityHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_source_policy_ledger_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_source_policy_ledger_availability_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilitySourceWriteAuthorityAvailabilityHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_source_write_authority_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_source_write_authority_availability_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilitySourceResultHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_source_result_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_source_result_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilitySourceLedgerCandidateHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_source_ledger_candidate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_source_ledger_candidate_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilitySourcePolicyPreflightHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilitySourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityAuditLedgerTargetId, "rollback_transaction_writer_storage_durable_audit_policy_availability_audit_ledger_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_audit_ledger_target_id),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityAuditRecordSchema, "rollback_transaction_writer_storage_durable_audit_policy_availability_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_audit_record_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityRollbackStoreTargetId, "rollback_transaction_writer_storage_durable_audit_policy_availability_rollback_store_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_rollback_store_target_id),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityRollbackTransactionSchema, "rollback_transaction_writer_storage_durable_audit_policy_availability_rollback_transaction_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_rollback_transaction_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityTargetStartLba, "rollback_transaction_writer_storage_durable_audit_policy_availability_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_target_start_lba),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityTargetLbaCount, "rollback_transaction_writer_storage_durable_audit_policy_availability_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_target_lba_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityTargetByteCount, "rollback_transaction_writer_storage_durable_audit_policy_availability_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_target_byte_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityPolicyLedgerAvailabilityEvidenceVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_policy_ledger_availability_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_policy_ledger_availability_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityWriteAuthorityEvidenceVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_write_authority_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_write_authority_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityLedgerEvidenceVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_ledger_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_ledger_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityMediaWritePolicyVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_media_write_policy_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_media_write_policy_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityTargetRegionWriteReadbackVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_target_region_write_readback_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_target_region_write_readback_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityTargetSpanVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_target_span_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_target_span_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityAuditRollbackTargetIdsVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_audit_rollback_target_ids_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_audit_rollback_target_ids_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_availability_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_availability_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDurablePolicyLedgerAvailable, "rollback_transaction_writer_storage_durable_audit_policy_availability_durable_policy_ledger_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_durable_policy_ledger_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_durable_audit_policy_availability_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_availability_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_durable_append_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityAuthorizesMediaWrite, "rollback_transaction_writer_storage_durable_audit_policy_availability_authorizes_media_write", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_authorizes_media_write),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityAuthorizesAppend, "rollback_transaction_writer_storage_durable_audit_policy_availability_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_authorizes_append),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_audit_policy_availability_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityWritesRollbackStore, "rollback_transaction_writer_storage_durable_audit_policy_availability_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_audit_policy_availability_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityWriteAttempted, "rollback_transaction_writer_storage_durable_audit_policy_availability_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_write_attempted),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunSchema, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunId, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_id),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunStatus, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_status),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunReason, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_reason),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunSourceAuditPolicyAvailabilityHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_audit_policy_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_audit_policy_availability_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunSourcePolicyLedgerAvailabilityDryRunHash,
        "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_policy_ledger_availability_dry_run_hash",
        BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_policy_ledger_availability_dry_run_hash),
        ),

    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunSourcePolicyLedgerAvailabilityHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_policy_ledger_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_policy_ledger_availability_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunSourceWriteAuthorityAvailabilityHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_write_authority_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_write_authority_availability_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunSourceResultHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_result_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_result_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunSourceLedgerCandidateHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_ledger_candidate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_ledger_candidate_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunSourcePolicyPreflightHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunSourceAuthorityDenialGateHash, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_authority_denial_gate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_authority_denial_gate_hash),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunSourceTransactionAppendAvailabilityDecisionHash,
        "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_transaction_append_availability_decision_hash",
        BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_source_transaction_append_availability_decision_hash),
        ),

    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunAuditLedgerTargetId, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_audit_ledger_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_audit_ledger_target_id),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunAuditRecordSchema, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_audit_record_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunRollbackStoreTargetId, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_rollback_store_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_rollback_store_target_id),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunRollbackTransactionSchema, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_rollback_transaction_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_rollback_transaction_schema),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunTargetStartLba, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_target_start_lba),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunTargetLbaCount, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_target_lba_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunTargetByteCount, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_target_byte_count),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunAuditPolicyAvailabilityEvidenceVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_audit_policy_availability_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_audit_policy_availability_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunPolicyLedgerDryRunEvidenceVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_policy_ledger_dry_run_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_policy_ledger_dry_run_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunPolicyLedgerAvailabilityEvidenceVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_policy_ledger_availability_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_policy_ledger_availability_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunWriteAuthorityEvidenceVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_write_authority_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_write_authority_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunLedgerEvidenceVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_ledger_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_ledger_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunMediaWritePolicyVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_media_write_policy_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_media_write_policy_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunTargetRegionWriteReadbackVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_target_region_write_readback_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_target_region_write_readback_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunTransactionAppendDenialGateVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_transaction_append_denial_gate_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_transaction_append_denial_gate_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunTargetSpanVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_target_span_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_target_span_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunAuditRollbackTargetIdsVerified, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_audit_rollback_target_ids_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_audit_rollback_target_ids_verified),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunDurablePolicyLedgerAvailable, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_durable_policy_ledger_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_durable_policy_ledger_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_durable_append_authority_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunTransactionAppendAvailable, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_transaction_append_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_transaction_append_available),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunAuthorizesMediaWrite, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_authorizes_media_write", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_authorizes_media_write),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunAuthorizesAppend, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_authorizes_append),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunAuthorizesTransactionAppend, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_authorizes_transaction_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_authorizes_transaction_append),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunWritesRollbackStore, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunWriteAttempted, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_write_attempted),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunAppliesRollback, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_applies_rollback", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_applies_rollback),),
    (RollbackTransactionWriterStorageDurableAuditPolicyAvailabilityDryRunInstallsRollbackState, "rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_installs_rollback_state", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_policy_availability_dry_run_installs_rollback_state),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilitySchema, "rollback_transaction_writer_storage_durable_append_authority_availability_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_schema),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityId, "rollback_transaction_writer_storage_durable_append_authority_availability_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_id),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityStatus, "rollback_transaction_writer_storage_durable_append_authority_availability_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_status),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityReason, "rollback_transaction_writer_storage_durable_append_authority_availability_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_reason),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityHash, "rollback_transaction_writer_storage_durable_append_authority_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilitySourceAuditPolicyAvailabilityHash, "rollback_transaction_writer_storage_durable_append_authority_availability_source_audit_policy_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_source_audit_policy_availability_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilitySourcePolicyLedgerAvailabilityHash, "rollback_transaction_writer_storage_durable_append_authority_availability_source_policy_ledger_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_source_policy_ledger_availability_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilitySourceWriteAuthorityAvailabilityHash, "rollback_transaction_writer_storage_durable_append_authority_availability_source_write_authority_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_source_write_authority_availability_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilitySourceResultHash, "rollback_transaction_writer_storage_durable_append_authority_availability_source_result_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_source_result_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilitySourceLedgerCandidateHash, "rollback_transaction_writer_storage_durable_append_authority_availability_source_ledger_candidate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_source_ledger_candidate_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilitySourcePolicyPreflightHash, "rollback_transaction_writer_storage_durable_append_authority_availability_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilitySourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_durable_append_authority_availability_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityAuditLedgerTargetId, "rollback_transaction_writer_storage_durable_append_authority_availability_audit_ledger_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_audit_ledger_target_id),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityAuditRecordSchema, "rollback_transaction_writer_storage_durable_append_authority_availability_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_audit_record_schema),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityRollbackStoreTargetId, "rollback_transaction_writer_storage_durable_append_authority_availability_rollback_store_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_rollback_store_target_id),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityRollbackTransactionSchema, "rollback_transaction_writer_storage_durable_append_authority_availability_rollback_transaction_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_rollback_transaction_schema),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityTargetStartLba, "rollback_transaction_writer_storage_durable_append_authority_availability_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_authority_availability_target_start_lba),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityTargetLbaCount, "rollback_transaction_writer_storage_durable_append_authority_availability_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_authority_availability_target_lba_count),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityTargetByteCount, "rollback_transaction_writer_storage_durable_append_authority_availability_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_authority_availability_target_byte_count),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityAuditPolicyAvailabilityEvidenceVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_audit_policy_availability_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_audit_policy_availability_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityPolicyLedgerAvailabilityEvidenceVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_policy_ledger_availability_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_policy_ledger_availability_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityWriteAuthorityEvidenceVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_write_authority_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_write_authority_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityLedgerEvidenceVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_ledger_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_ledger_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityMediaWritePolicyVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_media_write_policy_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_media_write_policy_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityTargetRegionWriteReadbackVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_target_region_write_readback_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_target_region_write_readback_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityTargetSpanVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_target_span_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_target_span_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityAuditRollbackTargetIdsVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_audit_rollback_target_ids_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_audit_rollback_target_ids_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_append_authority_availability_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_append_authority_availability_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDurablePolicyLedgerAvailable, "rollback_transaction_writer_storage_durable_append_authority_availability_durable_policy_ledger_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_durable_policy_ledger_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_durable_append_authority_availability_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_durable_append_authority_availability_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_durable_append_authority_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityAuthorizesMediaWrite, "rollback_transaction_writer_storage_durable_append_authority_availability_authorizes_media_write", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_authorizes_media_write),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityAuthorizesAppend, "rollback_transaction_writer_storage_durable_append_authority_availability_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_authorizes_append),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_append_authority_availability_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityWritesRollbackStore, "rollback_transaction_writer_storage_durable_append_authority_availability_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_append_authority_availability_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityWriteAttempted, "rollback_transaction_writer_storage_durable_append_authority_availability_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_write_attempted),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunSchema, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_schema),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunId, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_id),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunStatus, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_status),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunReason, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_reason),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunHash, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunSourceAppendAuthorityAvailabilityHash,
        "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_append_authority_availability_hash",
        BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_append_authority_availability_hash),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunSourceAuditPolicyAvailabilityDryRunHash,
        "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_audit_policy_availability_dry_run_hash",
        BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_audit_policy_availability_dry_run_hash),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunSourceAuditPolicyAvailabilityHash, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_audit_policy_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_audit_policy_availability_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunSourcePolicyLedgerAvailabilityDryRunHash,
        "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_policy_ledger_availability_dry_run_hash",
        BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_policy_ledger_availability_dry_run_hash),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunSourcePolicyLedgerAvailabilityHash, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_policy_ledger_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_policy_ledger_availability_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunSourceWriteAuthorityAvailabilityHash, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_write_authority_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_write_authority_availability_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunSourceResultHash, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_result_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_result_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunSourceLedgerCandidateHash, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_ledger_candidate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_ledger_candidate_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunSourcePolicyPreflightHash, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunSourceAuthorityDenialGateHash, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_authority_denial_gate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_authority_denial_gate_hash),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunSourceTransactionAppendAvailabilityDecisionHash,
        "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_transaction_append_availability_decision_hash",
        BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_source_transaction_append_availability_decision_hash),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunAuditLedgerTargetId, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_audit_ledger_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_audit_ledger_target_id),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunAuditRecordSchema, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_audit_record_schema),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunRollbackStoreTargetId, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_rollback_store_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_rollback_store_target_id),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunRollbackTransactionSchema, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_rollback_transaction_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_rollback_transaction_schema),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunTargetStartLba, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_target_start_lba),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunTargetLbaCount, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_target_lba_count),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunTargetByteCount, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_target_byte_count),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunAppendAuthorityAvailabilityEvidenceVerified,
        "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_append_authority_availability_evidence_verified",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_append_authority_availability_evidence_verified),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunAuditPolicyDryRunEvidenceVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_audit_policy_dry_run_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_audit_policy_dry_run_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunAuditPolicyAvailabilityEvidenceVerified,
        "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_audit_policy_availability_evidence_verified",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_audit_policy_availability_evidence_verified),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunPolicyLedgerDryRunEvidenceVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_policy_ledger_dry_run_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_policy_ledger_dry_run_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunPolicyLedgerAvailabilityEvidenceVerified,
        "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_policy_ledger_availability_evidence_verified",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_policy_ledger_availability_evidence_verified),
        ),

    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunWriteAuthorityEvidenceVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_write_authority_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_write_authority_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunLedgerEvidenceVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_ledger_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_ledger_evidence_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunMediaWritePolicyVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_media_write_policy_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_media_write_policy_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunTargetRegionWriteReadbackVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_target_region_write_readback_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_target_region_write_readback_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunTransactionAppendDenialGateVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_transaction_append_denial_gate_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_transaction_append_denial_gate_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunTargetSpanVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_target_span_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_target_span_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunAuditRollbackTargetIdsVerified, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_audit_rollback_target_ids_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_audit_rollback_target_ids_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_write_authority_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunDurablePolicyLedgerAvailable, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_durable_policy_ledger_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_durable_policy_ledger_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_durable_append_authority_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunTransactionAppendAvailable, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_transaction_append_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_transaction_append_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunAuthorizesMediaWrite, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_authorizes_media_write", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_authorizes_media_write),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunAuthorizesAppend, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_authorizes_append),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunAuthorizesTransactionAppend, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_authorizes_transaction_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_authorizes_transaction_append),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunWritesRollbackStore, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunWriteAttempted, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_write_attempted),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunAppliesRollback, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_applies_rollback", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_applies_rollback),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityAvailabilityDryRunInstallsRollbackState, "rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_installs_rollback_state", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_availability_dry_run_installs_rollback_state),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionSchema, "rollback_transaction_writer_storage_transaction_append_availability_decision_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_schema),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionId, "rollback_transaction_writer_storage_transaction_append_availability_decision_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_id),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionStatus, "rollback_transaction_writer_storage_transaction_append_availability_decision_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_status),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionReason, "rollback_transaction_writer_storage_transaction_append_availability_decision_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_reason),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionHash, "rollback_transaction_writer_storage_transaction_append_availability_decision_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_hash),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionSourceDurableAppendAuthorityAvailabilityHash,
        "rollback_transaction_writer_storage_transaction_append_availability_decision_source_durable_append_authority_availability_hash",
        BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_source_durable_append_authority_availability_hash),
        ),

    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionSourceAuditPolicyAvailabilityHash, "rollback_transaction_writer_storage_transaction_append_availability_decision_source_audit_policy_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_source_audit_policy_availability_hash),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionSourceAppendEngineReadinessDecisionHash, "rollback_transaction_writer_storage_transaction_append_availability_decision_source_append_engine_readiness_decision_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_source_append_engine_readiness_decision_hash),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionSourceWriterPolicyPreflightHash, "rollback_transaction_writer_storage_transaction_append_availability_decision_source_writer_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_source_writer_policy_preflight_hash),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionSourcePolicyPreflightHash, "rollback_transaction_writer_storage_transaction_append_availability_decision_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_transaction_append_availability_decision_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionAuditLedgerTargetId, "rollback_transaction_writer_storage_transaction_append_availability_decision_audit_ledger_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_audit_ledger_target_id),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionAuditRecordSchema, "rollback_transaction_writer_storage_transaction_append_availability_decision_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_audit_record_schema),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionRollbackStoreTargetId, "rollback_transaction_writer_storage_transaction_append_availability_decision_rollback_store_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_rollback_store_target_id),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionRollbackTransactionSchema, "rollback_transaction_writer_storage_transaction_append_availability_decision_rollback_transaction_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_rollback_transaction_schema),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionTargetStartLba, "rollback_transaction_writer_storage_transaction_append_availability_decision_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_target_start_lba),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionTargetLbaCount, "rollback_transaction_writer_storage_transaction_append_availability_decision_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_target_lba_count),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionTargetByteCount, "rollback_transaction_writer_storage_transaction_append_availability_decision_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_target_byte_count),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionDurableAppendAuthorityAvailabilityEvidenceVerified,
        "rollback_transaction_writer_storage_transaction_append_availability_decision_durable_append_authority_availability_evidence_verified",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_durable_append_authority_availability_evidence_verified),
        ),

    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionAuditPolicyAvailabilityEvidenceVerified, "rollback_transaction_writer_storage_transaction_append_availability_decision_audit_policy_availability_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_audit_policy_availability_evidence_verified),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionAppendEngineReady, "rollback_transaction_writer_storage_transaction_append_availability_decision_append_engine_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_append_engine_ready),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionWriterPolicyReady, "rollback_transaction_writer_storage_transaction_append_availability_decision_writer_policy_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_writer_policy_ready),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionMediaWritePolicyVerified, "rollback_transaction_writer_storage_transaction_append_availability_decision_media_write_policy_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_media_write_policy_verified),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionTargetRegionWriteReadbackVerified, "rollback_transaction_writer_storage_transaction_append_availability_decision_target_region_write_readback_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_target_region_write_readback_verified),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionTargetSpanVerified, "rollback_transaction_writer_storage_transaction_append_availability_decision_target_span_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_target_span_verified),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionAuditRollbackTargetIdsVerified, "rollback_transaction_writer_storage_transaction_append_availability_decision_audit_rollback_target_ids_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_audit_rollback_target_ids_verified),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_transaction_append_availability_decision_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_transaction_append_availability_decision_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_durable_append_authority_available),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_transaction_append_availability_decision_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionTransactionAppendAvailable, "rollback_transaction_writer_storage_transaction_append_availability_decision_transaction_append_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_transaction_append_available),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionAuthorizesMediaWrite, "rollback_transaction_writer_storage_transaction_append_availability_decision_authorizes_media_write", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_authorizes_media_write),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionAuthorizesAppend, "rollback_transaction_writer_storage_transaction_append_availability_decision_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_authorizes_append),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionAuthorizesTransactionAppend, "rollback_transaction_writer_storage_transaction_append_availability_decision_authorizes_transaction_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_authorizes_transaction_append),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionWritesDurableAuditLog, "rollback_transaction_writer_storage_transaction_append_availability_decision_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionWritesRollbackStore, "rollback_transaction_writer_storage_transaction_append_availability_decision_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_writes_rollback_store),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionAppendsRollbackTransaction, "rollback_transaction_writer_storage_transaction_append_availability_decision_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageTransactionAppendAvailabilityDecisionWriteAttempted, "rollback_transaction_writer_storage_transaction_append_availability_decision_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_availability_decision_write_attempted),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateSchema, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_schema),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateId, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_id),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateStatus, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_status),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateReason, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_reason),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateHash, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_hash),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateSourceTransactionAppendAvailabilityDecisionHash,
        "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_transaction_append_availability_decision_hash",
        BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_transaction_append_availability_decision_hash),
        ),

    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateSourceDurableAppendAuthorityAvailabilityHash,
        "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_durable_append_authority_availability_hash",
        BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_durable_append_authority_availability_hash),
        ),

    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateSourceAuditPolicyAvailabilityHash, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_audit_policy_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_audit_policy_availability_hash),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateSourceAppendEngineReadinessDecisionHash, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_append_engine_readiness_decision_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_append_engine_readiness_decision_hash),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateSourceWriterPolicyPreflightHash, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_writer_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_writer_policy_preflight_hash),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateSourcePolicyPreflightHash, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateAuditLedgerTargetId, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_audit_ledger_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_audit_ledger_target_id),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateAuditRecordSchema, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_audit_record_schema),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateRollbackStoreTargetId, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_rollback_store_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_rollback_store_target_id),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateRollbackTransactionSchema, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_rollback_transaction_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_rollback_transaction_schema),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateTargetStartLba, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_target_start_lba),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateTargetLbaCount, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_target_lba_count),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateTargetByteCount, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_target_byte_count),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateAvailabilityDecisionEvidenceVerified, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_availability_decision_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_availability_decision_evidence_verified),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateAppendEngineReady, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_append_engine_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_append_engine_ready),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateWriterPolicyReady, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_writer_policy_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_writer_policy_ready),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateMediaWritePolicyVerified, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_media_write_policy_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_media_write_policy_verified),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateTargetRegionWriteReadbackVerified, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_target_region_write_readback_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_target_region_write_readback_verified),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateTargetSpanVerified, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_target_span_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_target_span_verified),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateAuditRollbackTargetIdsVerified, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_audit_rollback_target_ids_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_audit_rollback_target_ids_verified),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_durable_append_authority_available),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateTransactionAppendAvailable, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_transaction_append_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_transaction_append_available),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateMissingTransactionAppendAuthority, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_missing_transaction_append_authority", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_missing_transaction_append_authority),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateAuthorizesMediaWrite, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_authorizes_media_write", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_authorizes_media_write),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateAuthorizesAppend, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_authorizes_append),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateAuthorizesTransactionAppend, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_authorizes_transaction_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_authorizes_transaction_append),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateWritesDurableAuditLog, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateWritesRollbackStore, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_writes_rollback_store),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateAppendsRollbackTransaction, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageTransactionAppendAuthorityDenialGateWriteAttempted, "rollback_transaction_writer_storage_transaction_append_authority_denial_gate_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_authority_denial_gate_write_attempted),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunSchema, "rollback_transaction_writer_storage_transaction_append_dry_run_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_dry_run_schema),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunId, "rollback_transaction_writer_storage_transaction_append_dry_run_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_dry_run_id),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunStatus, "rollback_transaction_writer_storage_transaction_append_dry_run_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_dry_run_status),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunReason, "rollback_transaction_writer_storage_transaction_append_dry_run_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_dry_run_reason),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunHash, "rollback_transaction_writer_storage_transaction_append_dry_run_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_dry_run_hash),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunSourceAuthorityDenialGateHash, "rollback_transaction_writer_storage_transaction_append_dry_run_source_authority_denial_gate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_dry_run_source_authority_denial_gate_hash),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunSourceTransactionAppendAvailabilityDecisionHash, "rollback_transaction_writer_storage_transaction_append_dry_run_source_transaction_append_availability_decision_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_dry_run_source_transaction_append_availability_decision_hash),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunSourceAppendRecordHash, "rollback_transaction_writer_storage_transaction_append_dry_run_source_append_record_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_dry_run_source_append_record_hash),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunSourceSectorPlanHash, "rollback_transaction_writer_storage_transaction_append_dry_run_source_sector_plan_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_dry_run_source_sector_plan_hash),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_transaction_append_dry_run_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_dry_run_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunPlannedSectorImageHash, "rollback_transaction_writer_storage_transaction_append_dry_run_planned_sector_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_dry_run_planned_sector_image_hash),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunReadbackSectorImageHash, "rollback_transaction_writer_storage_transaction_append_dry_run_readback_sector_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_transaction_append_dry_run_readback_sector_image_hash),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunAuditLedgerTargetId, "rollback_transaction_writer_storage_transaction_append_dry_run_audit_ledger_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_dry_run_audit_ledger_target_id),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunAuditRecordSchema, "rollback_transaction_writer_storage_transaction_append_dry_run_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_dry_run_audit_record_schema),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunRollbackStoreTargetId, "rollback_transaction_writer_storage_transaction_append_dry_run_rollback_store_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_dry_run_rollback_store_target_id),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunRollbackTransactionSchema, "rollback_transaction_writer_storage_transaction_append_dry_run_rollback_transaction_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_append_dry_run_rollback_transaction_schema),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunTargetStartLba, "rollback_transaction_writer_storage_transaction_append_dry_run_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_transaction_append_dry_run_target_start_lba),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunTargetLbaCount, "rollback_transaction_writer_storage_transaction_append_dry_run_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_transaction_append_dry_run_target_lba_count),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunTargetByteCount, "rollback_transaction_writer_storage_transaction_append_dry_run_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_transaction_append_dry_run_target_byte_count),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunAuthorityDenialGateVerified, "rollback_transaction_writer_storage_transaction_append_dry_run_authority_denial_gate_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_dry_run_authority_denial_gate_verified),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunTargetSpanVerified, "rollback_transaction_writer_storage_transaction_append_dry_run_target_span_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_dry_run_target_span_verified),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunTargetRegionWriteReadbackVerified, "rollback_transaction_writer_storage_transaction_append_dry_run_target_region_write_readback_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_dry_run_target_region_write_readback_verified),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunAppendImageReady, "rollback_transaction_writer_storage_transaction_append_dry_run_append_image_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_dry_run_append_image_ready),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunBlockedByAuthorityDenialGate, "rollback_transaction_writer_storage_transaction_append_dry_run_blocked_by_authority_denial_gate", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_dry_run_blocked_by_authority_denial_gate),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_transaction_append_dry_run_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_dry_run_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunTransactionAppendAvailable, "rollback_transaction_writer_storage_transaction_append_dry_run_transaction_append_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_dry_run_transaction_append_available),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunAuthorizesMediaWrite, "rollback_transaction_writer_storage_transaction_append_dry_run_authorizes_media_write", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_dry_run_authorizes_media_write),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunAuthorizesAppend, "rollback_transaction_writer_storage_transaction_append_dry_run_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_dry_run_authorizes_append),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunAuthorizesTransactionAppend, "rollback_transaction_writer_storage_transaction_append_dry_run_authorizes_transaction_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_dry_run_authorizes_transaction_append),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunWritesDurableAuditLog, "rollback_transaction_writer_storage_transaction_append_dry_run_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_dry_run_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunWritesRollbackStore, "rollback_transaction_writer_storage_transaction_append_dry_run_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_dry_run_writes_rollback_store),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunAppendsRollbackTransaction, "rollback_transaction_writer_storage_transaction_append_dry_run_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_dry_run_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageTransactionAppendDryRunTransactionAppendAttempted, "rollback_transaction_writer_storage_transaction_append_dry_run_transaction_append_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_append_dry_run_transaction_append_attempted),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionSchema, "rollback_transaction_writer_storage_target_region_sector_inspection_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_target_region_sector_inspection_schema),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionId, "rollback_transaction_writer_storage_target_region_sector_inspection_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_target_region_sector_inspection_id),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionStatus, "rollback_transaction_writer_storage_target_region_sector_inspection_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_target_region_sector_inspection_status),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionReason, "rollback_transaction_writer_storage_target_region_sector_inspection_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_target_region_sector_inspection_reason),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionHash, "rollback_transaction_writer_storage_target_region_sector_inspection_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_target_region_sector_inspection_hash),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionSourceSectorPlanHash, "rollback_transaction_writer_storage_target_region_sector_inspection_source_sector_plan_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_target_region_sector_inspection_source_sector_plan_hash),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_target_region_sector_inspection_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_target_region_sector_inspection_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionExpectedSectorImageHash, "rollback_transaction_writer_storage_target_region_sector_inspection_expected_sector_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_target_region_sector_inspection_expected_sector_image_hash),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionSectorImageHash, "rollback_transaction_writer_storage_target_region_sector_inspection_sector_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_target_region_sector_inspection_sector_image_hash),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionAuditRecordImageHash, "rollback_transaction_writer_storage_target_region_sector_inspection_audit_record_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_target_region_sector_inspection_audit_record_image_hash),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionRollbackTransactionImageHash, "rollback_transaction_writer_storage_target_region_sector_inspection_rollback_transaction_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_target_region_sector_inspection_rollback_transaction_image_hash),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionTargetStartLba, "rollback_transaction_writer_storage_target_region_sector_inspection_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_target_region_sector_inspection_target_start_lba),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionTargetLbaCount, "rollback_transaction_writer_storage_target_region_sector_inspection_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_target_region_sector_inspection_target_lba_count),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionTargetByteCount, "rollback_transaction_writer_storage_target_region_sector_inspection_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_target_region_sector_inspection_target_byte_count),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionAuditRecordOffset, "rollback_transaction_writer_storage_target_region_sector_inspection_audit_record_offset", BindingValue::OptU64(binding.rollback_transaction_writer_storage_target_region_sector_inspection_audit_record_offset),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionAuditRecordByteLength, "rollback_transaction_writer_storage_target_region_sector_inspection_audit_record_byte_length", BindingValue::OptU64(binding.rollback_transaction_writer_storage_target_region_sector_inspection_audit_record_byte_length),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionRollbackTransactionOffset, "rollback_transaction_writer_storage_target_region_sector_inspection_rollback_transaction_offset", BindingValue::OptU64(binding.rollback_transaction_writer_storage_target_region_sector_inspection_rollback_transaction_offset),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionRollbackTransactionByteLength, "rollback_transaction_writer_storage_target_region_sector_inspection_rollback_transaction_byte_length", BindingValue::OptU64(binding.rollback_transaction_writer_storage_target_region_sector_inspection_rollback_transaction_byte_length),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionPaddingOffset, "rollback_transaction_writer_storage_target_region_sector_inspection_padding_offset", BindingValue::OptU64(binding.rollback_transaction_writer_storage_target_region_sector_inspection_padding_offset),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionPaddingByteLength, "rollback_transaction_writer_storage_target_region_sector_inspection_padding_byte_length", BindingValue::OptU64(binding.rollback_transaction_writer_storage_target_region_sector_inspection_padding_byte_length),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionLabelFound, "rollback_transaction_writer_storage_target_region_sector_inspection_label_found", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_label_found),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionReadAttempted, "rollback_transaction_writer_storage_target_region_sector_inspection_read_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_read_attempted),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionReadCompleted, "rollback_transaction_writer_storage_target_region_sector_inspection_read_completed", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_read_completed),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionSectorHashVerified, "rollback_transaction_writer_storage_target_region_sector_inspection_sector_hash_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_sector_hash_verified),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionAuditRecordHashVerified, "rollback_transaction_writer_storage_target_region_sector_inspection_audit_record_hash_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_audit_record_hash_verified),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionRollbackTransactionHashVerified, "rollback_transaction_writer_storage_target_region_sector_inspection_rollback_transaction_hash_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_rollback_transaction_hash_verified),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionOffsetsVerified, "rollback_transaction_writer_storage_target_region_sector_inspection_offsets_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_offsets_verified),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionPaddingZeroed, "rollback_transaction_writer_storage_target_region_sector_inspection_padding_zeroed", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_padding_zeroed),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionTargetSpanVerified, "rollback_transaction_writer_storage_target_region_sector_inspection_target_span_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_target_span_verified),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionTargetRegionWriteReadbackVerified, "rollback_transaction_writer_storage_target_region_sector_inspection_target_region_write_readback_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_target_region_write_readback_verified),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionVerified, "rollback_transaction_writer_storage_target_region_sector_inspection_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_verified),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionAuthorizesMediaWrite, "rollback_transaction_writer_storage_target_region_sector_inspection_authorizes_media_write", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_authorizes_media_write),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionAuthorizesAppend, "rollback_transaction_writer_storage_target_region_sector_inspection_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_authorizes_append),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionWritesDurableAuditLog, "rollback_transaction_writer_storage_target_region_sector_inspection_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionWritesRollbackStore, "rollback_transaction_writer_storage_target_region_sector_inspection_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_writes_rollback_store),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionAppendsRollbackTransaction, "rollback_transaction_writer_storage_target_region_sector_inspection_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageTargetRegionSectorInspectionInstallsRollbackState, "rollback_transaction_writer_storage_target_region_sector_inspection_installs_rollback_state", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_sector_inspection_installs_rollback_state),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceSchema, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_schema),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceId, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_id),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceStatus, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_status),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceReason, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_reason),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceMethod, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_method", BindingValue::OptStr(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_method),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceEventId, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_event_id", BindingValue::OptEventId(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_event_id),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceAuditEventId, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_audit_event_id", BindingValue::OptEventId(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_audit_event_id),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceHash, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_hash),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceInspectionHash, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_inspection_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_inspection_hash),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceTargetRegionSectorInspectionHash, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_target_region_sector_inspection_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_target_region_sector_inspection_hash),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceSourceSectorPlanHash, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_source_sector_plan_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_source_sector_plan_hash),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceSourceTargetRegionWriteReadbackHash, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_source_target_region_write_readback_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_source_target_region_write_readback_hash),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceAvailable, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_available),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceMatchesSectorInspection, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_matches_sector_inspection", BindingValue::Bool(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_matches_sector_inspection),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceRamAuditStatus, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_ram_audit_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_ram_audit_status),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceRamAuditReason, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_ram_audit_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_ram_audit_reason),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceSourceEventRetained, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_source_event_retained", BindingValue::Bool(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_source_event_retained),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceAuditEventRetained, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_audit_event_retained", BindingValue::Bool(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_audit_event_retained),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceRamAuditValidated, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_ram_audit_validated", BindingValue::Bool(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_ram_audit_validated),),
    (RollbackTransactionWriterStorageRecoveryRollbackInspectSourceReferenceAuthorizesRollbackApply, "rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_authorizes_rollback_apply", BindingValue::Bool(binding.rollback_transaction_writer_storage_recovery_rollback_inspect_source_reference_authorizes_rollback_apply),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionSchema, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_schema),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionId, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_id),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionStatus, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_status),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionReason, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_reason),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionHash, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_hash),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionSourceDurableAppendAuthorityAvailabilityDryRunHash,
        "rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_durable_append_authority_availability_dry_run_hash",
        BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_durable_append_authority_availability_dry_run_hash),
        ),

    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionSourceTransactionAppendDryRunHash, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_transaction_append_dry_run_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_transaction_append_dry_run_hash),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionSourceTargetRegionSectorInspectionHash, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_target_region_sector_inspection_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_target_region_sector_inspection_hash),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionSourceWriteAuthorityAvailabilityHash, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_write_authority_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_write_authority_availability_hash),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionSourceAuditPolicyAvailabilityHash, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_audit_policy_availability_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_audit_policy_availability_hash),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionSourceDurableAppendAuthorityAvailabilityHash,
        "rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_durable_append_authority_availability_hash",
        BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_durable_append_authority_availability_hash),
        ),

    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionSourceAuthorityDenialGateHash, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_authority_denial_gate_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_authority_denial_gate_hash),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionSourceTransactionAppendAvailabilityDecisionHash,
        "rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_transaction_append_availability_decision_hash",
        BindingValue::OptSha256(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_source_transaction_append_availability_decision_hash),
        ),

    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionAuditLedgerTargetId, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_audit_ledger_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_audit_ledger_target_id),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionAuditRecordSchema, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_audit_record_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_audit_record_schema),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionRollbackStoreTargetId, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_rollback_store_target_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_rollback_store_target_id),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionRollbackTransactionSchema, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_rollback_transaction_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_rollback_transaction_schema),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionTargetStartLba, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_start_lba),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionTargetLbaCount, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_lba_count),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionTargetByteCount, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_byte_count),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionTransactionAppendDryRunVerified, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_transaction_append_dry_run_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_transaction_append_dry_run_verified),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionTargetRegionSectorInspectionVerified, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_region_sector_inspection_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_region_sector_inspection_verified),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionWriteAuthorityEvidenceVerified, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_write_authority_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_write_authority_evidence_verified),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionAuditPolicyAvailabilityEvidenceVerified, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_audit_policy_availability_evidence_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_audit_policy_availability_evidence_verified),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionDurableAppendAuthorityAvailabilityEvidenceVerified,
        "rollback_transaction_writer_storage_durable_policy_write_authority_decision_durable_append_authority_availability_evidence_verified",
        BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_durable_append_authority_availability_evidence_verified),
        ),

    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionTargetSpanVerified, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_span_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_target_span_verified),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionWriteAuthorityAvailable, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_write_authority_available),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionDurablePolicyLedgerAvailable, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_durable_policy_ledger_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_durable_policy_ledger_available),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionDurableAuditPolicyAvailable, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_durable_audit_policy_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_durable_audit_policy_available),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionDurableAppendAuthorityAvailable, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_durable_append_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_durable_append_authority_available),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionTransactionAppendAvailable, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_transaction_append_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_transaction_append_available),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionAuthorizesMediaWrite, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_authorizes_media_write", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_authorizes_media_write),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionAuthorizesAppend, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_authorizes_append),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionAuthorizesTransactionAppend, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_authorizes_transaction_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_authorizes_transaction_append),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionWritesRollbackStore, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionWriteAttempted, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_write_attempted),),
    (RollbackTransactionWriterStorageDurablePolicyWriteAuthorityDecisionAppliesRollback, "rollback_transaction_writer_storage_durable_policy_write_authority_decision_applies_rollback", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_policy_write_authority_decision_applies_rollback),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackDryRunSchema, "rollback_transaction_writer_storage_target_region_write_readback_dry_run_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_target_region_write_readback_dry_run_schema),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackDryRunId, "rollback_transaction_writer_storage_target_region_write_readback_dry_run_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_target_region_write_readback_dry_run_id),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackDryRunStatus, "rollback_transaction_writer_storage_target_region_write_readback_dry_run_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_target_region_write_readback_dry_run_status),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackDryRunReason, "rollback_transaction_writer_storage_target_region_write_readback_dry_run_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_target_region_write_readback_dry_run_reason),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackDryRunHash, "rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_target_region_write_readback_dry_run_hash),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackSourceSectorPlanHash, "rollback_transaction_writer_storage_target_region_write_readback_source_sector_plan_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_target_region_write_readback_source_sector_plan_hash),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackSourcePolicyPreflightHash, "rollback_transaction_writer_storage_target_region_write_readback_source_policy_preflight_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_target_region_write_readback_source_policy_preflight_hash),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackPlannedImageHash, "rollback_transaction_writer_storage_target_region_write_readback_planned_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_target_region_write_readback_planned_image_hash),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackReadbackImageHash, "rollback_transaction_writer_storage_target_region_write_readback_readback_image_hash", BindingValue::OptSha256(binding.rollback_transaction_writer_storage_target_region_write_readback_readback_image_hash),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackTargetStartLba, "rollback_transaction_writer_storage_target_region_write_readback_target_start_lba", BindingValue::OptU64(binding.rollback_transaction_writer_storage_target_region_write_readback_target_start_lba),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackTargetLbaCount, "rollback_transaction_writer_storage_target_region_write_readback_target_lba_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_target_region_write_readback_target_lba_count),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackTargetByteCount, "rollback_transaction_writer_storage_target_region_write_readback_target_byte_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_target_region_write_readback_target_byte_count),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackLabelFound, "rollback_transaction_writer_storage_target_region_write_readback_label_found", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_write_readback_label_found),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackTargetRangeReady, "rollback_transaction_writer_storage_target_region_write_readback_target_range_ready", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_write_readback_target_range_ready),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackTestMediaWriteAuthorityAvailable, "rollback_transaction_writer_storage_target_region_write_readback_test_media_write_authority_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_write_readback_test_media_write_authority_available),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackWriteAttempted, "rollback_transaction_writer_storage_target_region_write_readback_write_attempted", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_write_readback_write_attempted),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackWriteCompleted, "rollback_transaction_writer_storage_target_region_write_readback_write_completed", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_write_readback_write_completed),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackReadbackCompleted, "rollback_transaction_writer_storage_target_region_write_readback_readback_completed", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_write_readback_readback_completed),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackMatchesPlannedImage, "rollback_transaction_writer_storage_target_region_write_readback_matches_planned_image", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_write_readback_matches_planned_image),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackAuthorizesMediaWrite, "rollback_transaction_writer_storage_target_region_write_readback_authorizes_media_write", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_write_readback_authorizes_media_write),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackAuthorizesAppend, "rollback_transaction_writer_storage_target_region_write_readback_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_write_readback_authorizes_append),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackWritesDurableAuditLog, "rollback_transaction_writer_storage_target_region_write_readback_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_write_readback_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackWritesRollbackStore, "rollback_transaction_writer_storage_target_region_write_readback_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_write_readback_writes_rollback_store),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackAppendsRollbackTransaction, "rollback_transaction_writer_storage_target_region_write_readback_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_write_readback_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageTargetRegionWriteReadbackInstallsRollbackState, "rollback_transaction_writer_storage_target_region_write_readback_installs_rollback_state", BindingValue::Bool(binding.rollback_transaction_writer_storage_target_region_write_readback_installs_rollback_state),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionDiscoverySchema, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_schema", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_schema),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionDiscoveryId, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_id),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionDiscoveryStatus, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_status),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionDiscoveryReason, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_reason),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionDiscoverySource, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_source", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_discovery_source),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionPartitionInventoryScheme, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_partition_inventory_scheme", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_partition_inventory_scheme),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionPartitionEntryCount, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_partition_entry_count", BindingValue::OptU64(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_partition_entry_count),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionMbrSignatureValid, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_mbr_signature_valid", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_mbr_signature_valid),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionCandidatePresent, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_candidate_present", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_candidate_present),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionCandidateIsScratch, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_candidate_is_scratch", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_candidate_is_scratch),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionCandidateOverlapsBootMetadata, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_candidate_overlaps_boot_metadata", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_candidate_overlaps_boot_metadata),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionCandidateOverlapsScratch, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_candidate_overlaps_scratch", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_candidate_overlaps_scratch),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionScratchRejectedAsDurableAuthority, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_scratch_rejected_as_durable_authority", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_scratch_rejected_as_durable_authority),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTargetRegionDurableRegionAvailable, "rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_durable_region_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_target_region_durable_region_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightAuditWriterFactId, "rollback_transaction_writer_storage_durable_append_authority_preflight_audit_writer_fact_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_audit_writer_fact_id),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightRollbackWriterFactId, "rollback_transaction_writer_storage_durable_append_authority_preflight_rollback_writer_fact_id", BindingValue::OptStr(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_rollback_writer_fact_id),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightScratchWriteReadbackVerified, "rollback_transaction_writer_storage_durable_append_authority_preflight_scratch_write_readback_verified", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_scratch_write_readback_verified),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightScratchUsedAsDurableAuthority, "rollback_transaction_writer_storage_durable_append_authority_preflight_scratch_used_as_durable_authority", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_scratch_used_as_durable_authority),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightDurableAuditWriterAvailable, "rollback_transaction_writer_storage_durable_append_authority_preflight_durable_audit_writer_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_durable_audit_writer_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightRollbackStoreWriterAvailable, "rollback_transaction_writer_storage_durable_append_authority_preflight_rollback_store_writer_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_rollback_store_writer_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightTransactionAppendWriterAvailable, "rollback_transaction_writer_storage_durable_append_authority_preflight_transaction_append_writer_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_transaction_append_writer_available),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightAuthorizesAppend, "rollback_transaction_writer_storage_durable_append_authority_preflight_authorizes_append", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_authorizes_append),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightWritesDurableAuditLog, "rollback_transaction_writer_storage_durable_append_authority_preflight_writes_durable_audit_log", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_writes_durable_audit_log),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightWritesRollbackStore, "rollback_transaction_writer_storage_durable_append_authority_preflight_writes_rollback_store", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_writes_rollback_store),),
    (RollbackTransactionWriterStorageDurableAppendAuthorityPreflightAppendsRollbackTransaction, "rollback_transaction_writer_storage_durable_append_authority_preflight_appends_rollback_transaction", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_append_authority_preflight_appends_rollback_transaction),),
    (RollbackTransactionWriterStorageLayoutStatus, "rollback_transaction_writer_storage_layout_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_layout_status),),
    (RollbackTransactionWriterStorageLayoutReason, "rollback_transaction_writer_storage_layout_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_layout_reason),),
    (RollbackTransactionWriterStorageAppendEngineStatus, "rollback_transaction_writer_storage_append_engine_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_engine_status),),
    (RollbackTransactionWriterStorageAppendEngineReason, "rollback_transaction_writer_storage_append_engine_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_engine_reason),),
    (RollbackTransactionWriterStorageAppendContractStatus, "rollback_transaction_writer_storage_append_contract_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_contract_status),),
    (RollbackTransactionWriterStorageAppendContractReason, "rollback_transaction_writer_storage_append_contract_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_append_contract_reason),),
    (RollbackTransactionWriterStorageTransactionEnvelopeStatus, "rollback_transaction_writer_storage_transaction_envelope_status", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_envelope_status),),
    (RollbackTransactionWriterStorageTransactionEnvelopeReason, "rollback_transaction_writer_storage_transaction_envelope_reason", BindingValue::OptStr(binding.rollback_transaction_writer_storage_transaction_envelope_reason),),
    (RollbackTransactionWriterStorageTransactionWriterAvailable, "rollback_transaction_writer_storage_transaction_writer_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_transaction_writer_available),),
    (RollbackTransactionWriterStorageDurableAuditStoreAvailable, "rollback_transaction_writer_storage_durable_audit_store_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_durable_audit_store_available),),
    (RollbackTransactionWriterStorageRollbackStoreAvailable, "rollback_transaction_writer_storage_rollback_store_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_rollback_store_available),),
    (RollbackTransactionWriterStorageAppendAvailable, "rollback_transaction_writer_storage_append_available", BindingValue::Bool(binding.rollback_transaction_writer_storage_append_available),),
    (RollbackDurableAuditWriteAuthorityAvailable, "rollback_durable_audit_write_authority_available", BindingValue::Bool(binding.rollback_durable_audit_write_authority_available),),
    (RollbackStoreWriteAuthorityAvailable, "rollback_store_write_authority_available", BindingValue::Bool(binding.rollback_store_write_authority_available),),
    (RollbackTransactionAppendAvailable, "rollback_transaction_append_available", BindingValue::Bool(binding.rollback_transaction_append_available),),
    (BindsSourceLocator, "binds_source_locator", BindingValue::OptStr(binding.binds_source_locator),),
    (BindsSourceKind, "binds_source_kind", BindingValue::OptStr(binding.binds_source_kind),),
    (BindsSourceHash, "binds_source_hash", BindingValue::OptSha256(binding.binds_source_hash),),
    (LoadDescriptorSourceValidated, "load_descriptor_source_validated", BindingValue::Bool(binding.descriptor_source_validated),),
    (ServiceInventoryChange, "service_inventory_change", BindingValue::Str(binding.service_inventory_change),),
    (Persistence, "persistence", BindingValue::Str(binding.persistence),),
    (AcceptsExternalArtifactBytes, "accepts_external_artifact_bytes", BindingValue::Bool(binding.accepts_external_artifact_bytes),),
    (LoadsExternalArtifact, "loads_external_artifact", BindingValue::Bool(binding.loads_external_artifact),),
    (MapsExecutablePages, "maps_executable_pages", BindingValue::Bool(binding.maps_executable_pages),),
    (WritesPersistentState, "writes_persistent_state", BindingValue::Bool(binding.writes_persistent_state),),
    (SourceEvidenceRetained, "source_evidence_retained", BindingValue::Bool(true),),
    (Retention, "retention", BindingValue::Str("current_boot_ram_event_log"),),
}

fn hello_lifecycle_binding_schema(
    kind: &str,
    binding: &event_log::HelloServiceLifecycleBinding,
) -> &'static str {
    if binding.rollback_apply_authorized {
        "raios.ram_only_hello_service.rollback_apply_applied_binding.v0"
    } else if kind == "raios.ram_only_hello_service.health" {
        "raios.ram_only_hello_service.health_binding.v0"
    } else if kind == "raios.ram_only_hello_service.rollback_preview" {
        "raios.ram_only_hello_service.rollback_preview_binding.v0"
    } else if kind == "raios.ram_only_hello_service.rollback_apply" {
        "raios.ram_only_hello_service.rollback_apply_denial_binding.v0"
    } else {
        "raios.ram_only_hello_service.lifecycle_binding.v0"
    }
}

fn hello_lifecycle_binding_status(
    kind: &str,
    binding: &event_log::HelloServiceLifecycleBinding,
) -> &'static str {
    if binding.rollback_apply_authorized {
        "current_boot_rollback_applied"
    } else if kind == "raios.ram_only_hello_service.health" {
        "current_boot_health_read"
    } else if kind == "raios.ram_only_hello_service.rollback_preview" {
        "current_boot_rollback_preview_read"
    } else if kind == "raios.ram_only_hello_service.rollback_apply" {
        "current_boot_rollback_apply_denied"
    } else {
        "current_boot_state_transition"
    }
}

fn emit_hello_service_lifecycle_binding(
    kind: &str,
    binding: &event_log::HelloServiceLifecycleBinding,
) {
    emit_binding_object(
        binding,
        kind,
        HELLO_LIFECYCLE_BINDING_FIELDS,
        hello_lifecycle_binding_value,
    );
}

fn emit_event_bindings(kind: &str, bindings: &event_log::EventBindings) {
    match bindings {
        event_log::EventBindings::None => {}
        event_log::EventBindings::HelloServiceLifecycle(binding) => {
            emit_hello_service_lifecycle_binding(kind, binding);
        }
        event_log::EventBindings::HelloRecoveryRollbackInspectSourceReference(binding) => {
            emit_hello_recovery_rollback_inspect_source_reference_binding(kind, binding);
        }
        event_log::EventBindings::AgentCommandEnvelopeDecision(binding) => {
            emit_agent_command_envelope_decision_binding(kind, binding);
        }
        event_log::EventBindings::ProviderRequestEnvelope(binding) => {
            emit_provider_request_envelope_binding(kind, binding);
        }
        event_log::EventBindings::ProviderRequestBound(binding) => {
            emit_provider_request_bound_binding(kind, binding);
        }
        event_log::EventBindings::ProviderExportAuditBound(binding) => {
            emit_provider_export_audit_bound_binding(kind, binding);
        }
        event_log::EventBindings::ProviderBindingConsumption(binding) => {
            emit_provider_binding_consumption_binding(kind, binding);
        }
        event_log::EventBindings::ProviderContextInjectionAuthorization(binding) => {
            emit_provider_context_injection_authorization_binding(kind, binding);
        }
        event_log::EventBindings::ProviderRequestBindingDenied(hashes) => {
            emit_provider_request_binding_denied_binding(kind, hashes);
        }
        event_log::EventBindings::ProviderExportDenialAudit(hashes) => {
            emit_provider_export_denial_audit_binding(kind, hashes);
        }
        event_log::EventBindings::ModuleManifestReference(binding) => {
            emit_module_manifest_reference_binding(kind, binding);
        }
        event_log::EventBindings::ModuleCandidateArtifactReference(binding) => {
            emit_module_candidate_artifact_reference_binding(kind, binding);
        }
        event_log::EventBindings::ModuleVmTestReportReference(binding) => {
            emit_module_vm_test_report_reference_binding(kind, binding);
        }
        event_log::EventBindings::ModuleLocalAttestationReference(binding) => {
            emit_module_local_attestation_reference_binding(kind, binding);
        }
        event_log::EventBindings::ModulePromotionSignatureReference(_) => {}
        event_log::EventBindings::ModuleLocalApprovalReference(binding) => {
            emit_module_local_approval_reference_binding(kind, binding);
        }
        event_log::EventBindings::ModuleComputedGrantReference(binding) => {
            emit_module_computed_grant_reference_binding(kind, binding);
        }
        event_log::EventBindings::ModuleAuditRollbackReference(binding) => {
            emit_module_audit_rollback_reference_binding(kind, binding);
        }
        event_log::EventBindings::ModuleServiceSlotReservation(binding) => {
            emit_module_service_slot_reservation_binding(kind, binding);
        }
        event_log::EventBindings::ModuleServiceSlotAllocatorFactSourceEvidence(binding) => {
            emit_module_service_slot_allocator_fact_source_evidence_binding(kind, binding);
        }
        event_log::EventBindings::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence(binding) => {
            emit_module_service_slot_allocator_prerequisite_source_evidence_binding(kind, binding);
        }
        event_log::EventBindings::ModuleServiceSlotAllocatorAuthoritySourceEvidence(binding) => {
            emit_module_service_slot_allocator_authority_source_evidence_binding(kind, binding);
        }
        event_log::EventBindings::ModuleServiceSlotAllocationIntentSourceEvidence(binding) => {
            emit_module_service_slot_allocation_intent_source_evidence_binding(kind, binding);
        }
        event_log::EventBindings::ModuleServiceSlotAuthorityInputSourceEvidence(binding) => {
            emit_module_service_slot_authority_input_source_evidence_binding(kind, binding);
        }
        event_log::EventBindings::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence(binding) => {
            emit_module_service_slot_allocator_authority_decision_source_evidence_binding(kind, binding);
        }
        event_log::EventBindings::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence(binding) => {
            emit_module_service_slot_registry_write_commit_gate_source_evidence_binding(kind, binding);
        }
        event_log::EventBindings::ModuleLoaderIdentitySourceEvidence(binding) => {
            emit_module_loader_identity_source_evidence_binding(kind, binding);
        }
        event_log::EventBindings::ModuleLoaderArtifactHashBindingSourceEvidence(binding) => {
            emit_module_loader_artifact_hash_binding_source_evidence_binding(kind, binding);
        }
        event_log::EventBindings::ModuleLoaderFactSourceEvidence(binding) => {
            emit_module_loader_fact_source_evidence_binding(kind, binding);
        }
        event_log::EventBindings::ModuleLoaderRuntimeExecutionCommitGateSourceEvidence(binding) => {
            emit_module_loader_runtime_execution_commit_gate_source_evidence_binding(kind, binding);
        }
        event_log::EventBindings::ModuleLoaderDescriptorIntakeBoundarySourceEvidence(binding) => {
            emit_module_loader_descriptor_intake_boundary_source_evidence_binding(kind, binding);
        }
        event_log::EventBindings::ModuleLoaderArtifactByteIntakeBoundarySourceEvidence(binding) => {
            emit_module_loader_artifact_byte_intake_boundary_source_evidence_binding(kind, binding);
        }
        event_log::EventBindings::ModuleLoaderExecutionAuthorizationBoundarySourceEvidence(binding) => {
            emit_module_loader_execution_authorization_boundary_source_evidence_binding(kind, binding);
        }
        event_log::EventBindings::ModuleLoaderServiceRegistryMutationBoundarySourceEvidence(binding) => {
            emit_module_loader_service_registry_mutation_boundary_source_evidence_binding(kind, binding);
        }
        event_log::EventBindings::ModuleLoaderLoadAttemptBoundarySourceEvidence(binding)
        | event_log::EventBindings::ModuleLoaderArtifactLoadBoundarySourceEvidence(binding)
        | event_log::EventBindings::ModuleLoaderExecutableMappingBoundarySourceEvidence(binding)
        | event_log::EventBindings::ModuleLoaderEntrypointTransferBoundarySourceEvidence(binding)
        | event_log::EventBindings::ModuleLoaderServiceStartBoundarySourceEvidence(binding)
        | event_log::EventBindings::ModuleLoaderServiceHealthBindingBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderServiceRunningStateBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderServiceStartAuditBoundarySourceEvidence(binding)
        | event_log::EventBindings::ModuleLoaderServiceUnloadCleanupBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderLiveLoadCommitBoundarySourceEvidence(binding)
        | event_log::EventBindings::ModuleLoaderCommitAuditBoundarySourceEvidence(binding)
        | event_log::EventBindings::ModuleLoaderCommitRollbackBoundarySourceEvidence(binding)
        | event_log::EventBindings::ModuleLoaderCommitResultBoundarySourceEvidence(binding)
        | event_log::EventBindings::ModuleLoaderDescriptorAcceptanceAuthorityBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderDescriptorParserContractBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderDescriptorParserResultBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderDescriptorSchemaValidationBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderDescriptorCapabilityValidationBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderDescriptorLoadPlanBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderExecutableLoadPlanAuthorityBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderExecutableLoadPlanResultBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderExecutableImageLayoutBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderExecutablePageMappingPlanBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderExecutablePageMappingBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderDescriptorExecutablePageBindingBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderExecutableEntrypointBindingBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderExecutableEntrypointTransferAuthorizationBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderExecutableEntrypointTransferBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderExecutableEntrypointHandoffBoundarySourceEvidence(
            binding,
        )
        | event_log::EventBindings::ModuleLoaderExecutableEntrypointInvocationBoundarySourceEvidence(
            binding,
        ) => {
            emit_module_loader_live_load_boundary_binding(kind, binding);
        }
        event_log::EventBindings::ModuleLoadGate(binding) => {
            emit_module_load_gate_event_binding(*binding);
        }
        event_log::EventBindings::RecoveryArtifactIdentityReference(binding) => {
            emit_recovery_artifact_identity_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryArtifactTrustReference(binding) => {
            emit_recovery_artifact_trust_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryArtifactVmTestReference(binding) => {
            emit_recovery_artifact_vm_test_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryArtifactLocalApprovalReference(binding) => {
            emit_recovery_artifact_local_approval_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryArtifactLoaderReference(binding) => {
            emit_recovery_artifact_loader_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryArtifactRollbackEvidenceReference(binding) => {
            emit_recovery_artifact_rollback_evidence_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryLifelineRequestReference(binding) => {
            emit_recovery_lifeline_request_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryLifelineCommandEnvelopeReference(binding) => {
            emit_recovery_lifeline_command_envelope_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryLifelineCommandBodyCanonicalizationReference(binding) => {
            emit_recovery_lifeline_command_body_canonicalization_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryLifelineCommandHandlerBindingReference(binding) => {
            emit_recovery_lifeline_command_handler_binding_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryLifelineStatusReadHandlerReference(binding) => {
            emit_recovery_lifeline_status_read_handler_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryRollbackPreviewAuthorizationReference(binding) => {
            emit_recovery_rollback_preview_authorization_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryRollbackApplyAuthorizationReference(binding) => {
            emit_recovery_rollback_apply_authorization_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryDisableModuleTargetBindingReference(binding) => {
            emit_recovery_disable_module_target_binding_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryRestartLastGoodTargetBindingReference(binding) => {
            emit_recovery_restart_last_good_target_binding_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryLoadArtifactByHashTargetBindingReference(binding) => {
            emit_recovery_load_artifact_by_hash_target_binding_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryMemoryWriteAuthorityReference(binding) => {
            emit_recovery_memory_write_authority_reference_binding(kind, binding);
        }
        event_log::EventBindings::DurableAuditRollbackWriteAuthorityReference(binding) => {
            emit_durable_audit_rollback_write_authority_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryServiceInventorySideEffectBoundaryReference(binding) => {
            emit_recovery_service_inventory_side_effect_boundary_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryLifelineCommandDispatchBehaviorReference(binding) => {
            emit_recovery_lifeline_command_dispatch_behavior_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryLifelineCommandExecutorCapabilityTableReference(binding) => {
            emit_recovery_lifeline_command_executor_capability_table_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryLifelineCommandSideEffectGateReference(binding) => {
            emit_recovery_lifeline_command_side_effect_gate_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryLifelineCommandExecutionStageReference(binding) => {
            emit_recovery_lifeline_command_execution_stage_reference_binding(kind, binding);
        }
        event_log::EventBindings::RecoveryLifelineStatusExecutionResultReference(binding) => {
            emit_recovery_lifeline_status_execution_result_reference_binding(kind, binding);
        }
    }
}

define_direct_binding_fields! { ModuleLoaderLiveLoadBoundaryBindingField,
MODULE_LOADER_LIVE_LOAD_BOUNDARY_BINDING_FIELDS,
emit_module_loader_live_load_boundary_binding_value,
event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(BoundarySchema,
"boundary_schema",
{ json_str(binding.boundary_schema);
}),
(BoundaryId,
"boundary_id",
{ json_str(binding.boundary_id);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ json_str(binding.requested_capability);
}),
(LoadMode,
"load_mode",
{ json_str(binding.load_mode);
}),
(Target,
"target",
{ json_str(binding.target);
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(BoundaryStatus,
"boundary_status",
{ json_str(binding.boundary_status);
}),
(BoundaryReason,
"boundary_reason",
{ json_str(binding.boundary_reason);
}),
(BoundaryPresent,
"boundary_present",
{ raw_bool(binding.boundary_present);
}),
(BoundaryScope,
"boundary_scope",
{ json_str(binding.boundary_scope);
}),
(BoundarySchemaOk,
"boundary_schema_ok",
{ raw_bool(binding.boundary_schema_ok);
}),
(BoundaryProvenanceOk,
"boundary_provenance_ok",
{ raw_bool(binding.boundary_provenance_ok);
}),
(BoundaryClassification,
"boundary_classification",
{ json_str(binding.boundary_classification);
}),
(SourceChainComplete,
"source_chain_complete",
{ raw_bool(binding.source_chain_complete);
}),
(LoadAttemptBoundaryPresent,
"load_attempt_boundary_present",
{ raw_bool(binding.load_attempt_boundary_present);
}),
(ArtifactLoadBoundaryPresent,
"artifact_load_boundary_present",
{ raw_bool(binding.artifact_load_boundary_present);
}),
(ExecutableMappingBoundaryPresent,
"executable_mapping_boundary_present",
{ raw_bool(binding.executable_mapping_boundary_present);
}),
(EntrypointTransferBoundaryPresent,
"entrypoint_transfer_boundary_present",
{ raw_bool(binding.entrypoint_transfer_boundary_present);
}),
(ServiceStartBoundaryPresent,
"service_start_boundary_present",
{ raw_bool(binding.service_start_boundary_present);
}),
(ServiceHealthBindingBoundaryPresent,
"service_health_binding_boundary_present",
{ raw_bool(binding.service_health_binding_boundary_present);
}),
(ServiceRunningStateBoundaryPresent,
"service_running_state_boundary_present",
{ raw_bool(binding.service_running_state_boundary_present);
}),
(ServiceStartAuditBoundaryPresent,
"service_start_audit_boundary_present",
{ raw_bool(binding.service_start_audit_boundary_present);
}),
(ServiceUnloadCleanupBoundaryPresent,
"service_unload_cleanup_boundary_present",
{ raw_bool(binding.service_unload_cleanup_boundary_present);
}),
(LiveLoadCommitBoundaryPresent,
"live_load_commit_boundary_present",
{ raw_bool(binding.live_load_commit_boundary_present);
}),
(CommitAuditBoundaryPresent,
"commit_audit_boundary_present",
{ raw_bool(binding.commit_audit_boundary_present);
}),
(CommitRollbackBoundaryPresent,
"commit_rollback_boundary_present",
{ raw_bool(binding.commit_rollback_boundary_present);
}),
(CommitResultBoundaryPresent,
"commit_result_boundary_present",
{ raw_bool(binding.commit_result_boundary_present);
}),
(DescriptorAcceptanceAuthorityBoundaryPresent,
"descriptor_acceptance_authority_boundary_present",
{ raw_bool(binding.descriptor_acceptance_authority_boundary_present);
}),
(DescriptorParserContractBoundaryPresent,
"descriptor_parser_contract_boundary_present",
{ raw_bool(binding.descriptor_parser_contract_boundary_present);
}),
(DescriptorParserResultBoundaryPresent,
"descriptor_parser_result_boundary_present",
{ raw_bool(binding.descriptor_parser_result_boundary_present);
}),
(DescriptorSchemaValidationBoundaryPresent,
"descriptor_schema_validation_boundary_present",
{ raw_bool(binding.descriptor_schema_validation_boundary_present);
}),
(DescriptorSchemaValidationBoundarySourceChainComplete,
"descriptor_schema_validation_boundary_source_chain_complete",
{ raw_bool(binding.descriptor_schema_validation_boundary_source_chain_complete);
}),
(DescriptorCapabilityValidationBoundaryPresent,
"descriptor_capability_validation_boundary_present",
{ raw_bool(binding.descriptor_capability_validation_boundary_present);
}),
(DescriptorCapabilityValidationBoundarySourceChainComplete,
"descriptor_capability_validation_boundary_source_chain_complete",
{ raw_bool(binding.descriptor_capability_validation_boundary_source_chain_complete);
}),
(DescriptorLoadPlanBoundaryPresent,
"descriptor_load_plan_boundary_present",
{ raw_bool(binding.descriptor_load_plan_boundary_present);
}),
(DescriptorLoadPlanBoundarySourceChainComplete,
"descriptor_load_plan_boundary_source_chain_complete",
{ raw_bool(binding.descriptor_load_plan_boundary_source_chain_complete);
}),
(ExecutableLoadPlanAuthorityBoundaryPresent,
"executable_load_plan_authority_boundary_present",
{ raw_bool(binding.executable_load_plan_authority_boundary_present);
}),
(ExecutableLoadPlanAuthorityBoundarySourceChainComplete,
"executable_load_plan_authority_boundary_source_chain_complete",
{ raw_bool(binding.executable_load_plan_authority_boundary_source_chain_complete);
}),
(ExecutableLoadPlanResultBoundaryPresent,
"executable_load_plan_result_boundary_present",
{ raw_bool(binding.executable_load_plan_result_boundary_present);
}),
(ExecutableLoadPlanResultBoundarySourceChainComplete,
"executable_load_plan_result_boundary_source_chain_complete",
{ raw_bool(binding.executable_load_plan_result_boundary_source_chain_complete);
}),
(ExecutableImageLayoutBoundaryPresent,
"executable_image_layout_boundary_present",
{ raw_bool(binding.executable_image_layout_boundary_present);
}),
(ExecutableImageLayoutBoundarySourceChainComplete,
"executable_image_layout_boundary_source_chain_complete",
{ raw_bool(binding.executable_image_layout_boundary_source_chain_complete);
}),
(ExecutablePageMappingPlanBoundaryPresent,
"executable_page_mapping_plan_boundary_present",
{ raw_bool(binding.executable_page_mapping_plan_boundary_present);
}),
(ExecutablePageMappingPlanBoundarySourceChainComplete,
"executable_page_mapping_plan_boundary_source_chain_complete",
{ raw_bool(binding.executable_page_mapping_plan_boundary_source_chain_complete);
}),
(ExecutablePageMappingBoundaryPresent,
"executable_page_mapping_boundary_present",
{ raw_bool(binding.executable_page_mapping_boundary_present);
}),
(ExecutablePageMappingBoundarySourceChainComplete,
"executable_page_mapping_boundary_source_chain_complete",
{ raw_bool(binding.executable_page_mapping_boundary_source_chain_complete);
}),
(DescriptorExecutablePageBindingBoundaryPresent,
"descriptor_executable_page_binding_boundary_present",
{ raw_bool(binding.descriptor_executable_page_binding_boundary_present);
}),
(DescriptorExecutablePageBindingBoundarySourceChainComplete,
"descriptor_executable_page_binding_boundary_source_chain_complete",
{ raw_bool(binding.descriptor_executable_page_binding_boundary_source_chain_complete);
}),
(ExecutableEntrypointBindingBoundaryPresent,
"executable_entrypoint_binding_boundary_present",
{ raw_bool(binding.executable_entrypoint_binding_boundary_present);
}),
(ExecutableEntrypointBindingBoundarySourceChainComplete,
"executable_entrypoint_binding_boundary_source_chain_complete",
{ raw_bool(binding.executable_entrypoint_binding_boundary_source_chain_complete);
}),
(ExecutableEntrypointTransferAuthorizationBoundaryPresent,
"executable_entrypoint_transfer_authorization_boundary_present",
{ raw_bool(binding.executable_entrypoint_transfer_authorization_boundary_present);
}),
(ExecutableEntrypointTransferAuthorizationBoundarySourceChainComplete,
"executable_entrypoint_transfer_authorization_boundary_source_chain_complete",
{ raw_bool(binding.executable_entrypoint_transfer_authorization_boundary_source_chain_complete);
}),
(ExecutableEntrypointTransferBoundaryPresent,
"executable_entrypoint_transfer_boundary_present",
{ raw_bool(binding.executable_entrypoint_transfer_boundary_present);
}),
(ExecutableEntrypointTransferBoundarySourceChainComplete,
"executable_entrypoint_transfer_boundary_source_chain_complete",
{ raw_bool(binding.executable_entrypoint_transfer_boundary_source_chain_complete);
}),
(ExecutableEntrypointHandoffBoundaryPresent,
"executable_entrypoint_handoff_boundary_present",
{ raw_bool(binding.executable_entrypoint_handoff_boundary_present);
}),
(ExecutableEntrypointHandoffBoundarySourceChainComplete,
"executable_entrypoint_handoff_boundary_source_chain_complete",
{ raw_bool(binding.executable_entrypoint_handoff_boundary_source_chain_complete);
}),
(ArtifactByteIntakeBoundaryPresent,
"artifact_byte_intake_boundary_present",
{ raw_bool(binding.artifact_byte_intake_boundary_present);
}),
(ExecutionAuthorizationBoundaryPresent,
"execution_authorization_boundary_present",
{ raw_bool(binding.execution_authorization_boundary_present);
}),
(ServiceRegistryMutationBoundaryPresent,
"service_registry_mutation_boundary_present",
{ raw_bool(binding.service_registry_mutation_boundary_present);
}),
(ServiceSlotBindingSourceEvidencePresent,
"service_slot_binding_source_evidence_present",
{ raw_bool(binding.service_slot_binding_source_evidence_present);
}),
(HealthStateHooksSourceEvidencePresent,
"health_state_hooks_source_evidence_present",
{ raw_bool(binding.health_state_hooks_source_evidence_present);
}),
(ArtifactHashBindingPresent,
"artifact_hash_binding_present",
{ raw_bool(binding.artifact_hash_binding_present);
}),
(EntrypointAbiSourceEvidencePresent,
"entrypoint_abi_source_evidence_present",
{ raw_bool(binding.entrypoint_abi_source_evidence_present);
}),
(AddressSpaceSourceEvidencePresent,
"address_space_source_evidence_present",
{ raw_bool(binding.address_space_source_evidence_present);
}),
(MemoryMapSourceEvidencePresent,
"memory_map_source_evidence_present",
{ raw_bool(binding.memory_map_source_evidence_present);
}),
(CapabilityImportTableSourceEvidencePresent,
"capability_import_table_source_evidence_present",
{ raw_bool(binding.capability_import_table_source_evidence_present);
}),
(AuditRollbackWriteBoundarySourceEvidencePresent,
"audit_rollback_write_boundary_source_evidence_present",
{ raw_bool(binding.audit_rollback_write_boundary_source_evidence_present);
}),
(RetainedModuleEvidencePresent,
"retained_module_evidence_present",
{ raw_bool(binding.retained_module_evidence_present);
}),
(RetainedArtifactReferencePresent,
"retained_artifact_reference_present",
{ raw_bool(binding.retained_artifact_reference_present);
}),
(RetainedServiceSlotReservationPresent,
"retained_service_slot_reservation_present",
{ raw_bool(binding.retained_service_slot_reservation_present);
}),
(LoadAttemptBoundarySourceEvidenceEventId,
"load_attempt_boundary_source_evidence_event_id",
{ json_event_id_option(binding.load_attempt_boundary_source_evidence_event_id);
}),
(ArtifactLoadBoundarySourceEvidenceEventId,
"artifact_load_boundary_source_evidence_event_id",
{ json_event_id_option(binding.artifact_load_boundary_source_evidence_event_id);
}),
(ExecutableMappingBoundarySourceEvidenceEventId,
"executable_mapping_boundary_source_evidence_event_id",
{ json_event_id_option(binding.executable_mapping_boundary_source_evidence_event_id);
}),
(EntrypointTransferBoundarySourceEvidenceEventId,
"entrypoint_transfer_boundary_source_evidence_event_id",
{ json_event_id_option(binding.entrypoint_transfer_boundary_source_evidence_event_id);
}),
(ServiceStartBoundarySourceEvidenceEventId,
"service_start_boundary_source_evidence_event_id",
{ json_event_id_option(binding.service_start_boundary_source_evidence_event_id);
}),
(ServiceHealthBindingBoundarySourceEvidenceEventId,
"service_health_binding_boundary_source_evidence_event_id",
{ json_event_id_option(binding.service_health_binding_boundary_source_evidence_event_id);
}),
(ServiceRunningStateBoundarySourceEvidenceEventId,
"service_running_state_boundary_source_evidence_event_id",
{ json_event_id_option(binding.service_running_state_boundary_source_evidence_event_id);
}),
(ServiceStartAuditBoundarySourceEvidenceEventId,
"service_start_audit_boundary_source_evidence_event_id",
{ json_event_id_option(binding.service_start_audit_boundary_source_evidence_event_id);
}),
(ServiceUnloadCleanupBoundarySourceEvidenceEventId,
"service_unload_cleanup_boundary_source_evidence_event_id",
{ json_event_id_option(binding.service_unload_cleanup_boundary_source_evidence_event_id);
}),
(LiveLoadCommitBoundarySourceEvidenceEventId,
"live_load_commit_boundary_source_evidence_event_id",
{ json_event_id_option(binding.live_load_commit_boundary_source_evidence_event_id);
}),
(CommitAuditBoundarySourceEvidenceEventId,
"commit_audit_boundary_source_evidence_event_id",
{ json_event_id_option(binding.commit_audit_boundary_source_evidence_event_id);
}),
(CommitRollbackBoundarySourceEvidenceEventId,
"commit_rollback_boundary_source_evidence_event_id",
{ json_event_id_option(binding.commit_rollback_boundary_source_evidence_event_id);
}),
(CommitResultBoundarySourceEvidenceEventId,
"commit_result_boundary_source_evidence_event_id",
{ json_event_id_option(binding.commit_result_boundary_source_evidence_event_id);
}),
(DescriptorAcceptanceAuthorityBoundarySourceEvidenceEventId,
"descriptor_acceptance_authority_boundary_source_evidence_event_id",
{ json_event_id_option(binding.descriptor_acceptance_authority_boundary_source_evidence_event_id);
}),
(DescriptorParserContractBoundarySourceEvidenceEventId,
"descriptor_parser_contract_boundary_source_evidence_event_id",
{ json_event_id_option(binding.descriptor_parser_contract_boundary_source_evidence_event_id);
}),
(DescriptorParserResultBoundarySourceEvidenceEventId,
"descriptor_parser_result_boundary_source_evidence_event_id",
{ json_event_id_option(binding.descriptor_parser_result_boundary_source_evidence_event_id);
}),
(DescriptorSchemaValidationBoundarySourceEvidenceEventId,
"descriptor_schema_validation_boundary_source_evidence_event_id",
{ json_event_id_option(binding.descriptor_schema_validation_boundary_source_evidence_event_id);
}),
(DescriptorCapabilityValidationBoundarySourceEvidenceEventId,
"descriptor_capability_validation_boundary_source_evidence_event_id",
{ json_event_id_option( binding.descriptor_capability_validation_boundary_source_evidence_event_id,
);
}),
(DescriptorLoadPlanBoundarySourceEvidenceEventId,
"descriptor_load_plan_boundary_source_evidence_event_id",
{ json_event_id_option(binding.descriptor_load_plan_boundary_source_evidence_event_id);
}),
(ExecutableLoadPlanAuthorityBoundarySourceEvidenceEventId,
"executable_load_plan_authority_boundary_source_evidence_event_id",
{ json_event_id_option(binding.executable_load_plan_authority_boundary_source_evidence_event_id);
}),
(ExecutableLoadPlanResultBoundarySourceEvidenceEventId,
"executable_load_plan_result_boundary_source_evidence_event_id",
{ json_event_id_option(binding.executable_load_plan_result_boundary_source_evidence_event_id);
}),
(ExecutableImageLayoutBoundarySourceEvidenceEventId,
"executable_image_layout_boundary_source_evidence_event_id",
{ json_event_id_option(binding.executable_image_layout_boundary_source_evidence_event_id);
}),
(ExecutablePageMappingPlanBoundarySourceEvidenceEventId,
"executable_page_mapping_plan_boundary_source_evidence_event_id",
{ json_event_id_option(binding.executable_page_mapping_plan_boundary_source_evidence_event_id);
}),
(ExecutablePageMappingBoundarySourceEvidenceEventId,
"executable_page_mapping_boundary_source_evidence_event_id",
{ json_event_id_option(binding.executable_page_mapping_boundary_source_evidence_event_id);
}),
(DescriptorExecutablePageBindingBoundarySourceEvidenceEventId,
"descriptor_executable_page_binding_boundary_source_evidence_event_id",
{ json_event_id_option( binding.descriptor_executable_page_binding_boundary_source_evidence_event_id,
);
}),
(ExecutableEntrypointBindingBoundarySourceEvidenceEventId,
"executable_entrypoint_binding_boundary_source_evidence_event_id",
{ json_event_id_option(binding.executable_entrypoint_binding_boundary_source_evidence_event_id);
}),
(ExecutableEntrypointTransferAuthorizationBoundarySourceEvidenceEventId,
"executable_entrypoint_transfer_authorization_boundary_source_evidence_event_id",
{ json_event_id_option( binding.executable_entrypoint_transfer_authorization_boundary_source_evidence_event_id,
);
}),
(ExecutableEntrypointTransferBoundarySourceEvidenceEventId,
"executable_entrypoint_transfer_boundary_source_evidence_event_id",
{ json_event_id_option(binding.executable_entrypoint_transfer_boundary_source_evidence_event_id);
}),
(ExecutableEntrypointHandoffBoundarySourceEvidenceEventId,
"executable_entrypoint_handoff_boundary_source_evidence_event_id",
{ json_event_id_option(binding.executable_entrypoint_handoff_boundary_source_evidence_event_id);
}),
(ArtifactByteIntakeBoundarySourceEvidenceEventId,
"artifact_byte_intake_boundary_source_evidence_event_id",
{ json_event_id_option(binding.artifact_byte_intake_boundary_source_evidence_event_id);
}),
(ExecutionAuthorizationBoundarySourceEvidenceEventId,
"execution_authorization_boundary_source_evidence_event_id",
{ json_event_id_option(binding.execution_authorization_boundary_source_evidence_event_id);
}),
(ServiceRegistryMutationBoundarySourceEvidenceEventId,
"service_registry_mutation_boundary_source_evidence_event_id",
{ json_event_id_option(binding.service_registry_mutation_boundary_source_evidence_event_id);
}),
(ServiceSlotBindingSourceEvidenceEventId,
"service_slot_binding_source_evidence_event_id",
{ json_event_id_option(binding.service_slot_binding_source_evidence_event_id);
}),
(HealthStateHooksSourceEvidenceEventId,
"health_state_hooks_source_evidence_event_id",
{ json_event_id_option(binding.health_state_hooks_source_evidence_event_id);
}),
(ArtifactHashBindingSourceEvidenceEventId,
"artifact_hash_binding_source_evidence_event_id",
{ json_event_id_option(binding.artifact_hash_binding_source_evidence_event_id);
}),
(EntrypointAbiSourceEvidenceEventId,
"entrypoint_abi_source_evidence_event_id",
{ json_event_id_option(binding.entrypoint_abi_source_evidence_event_id);
}),
(AddressSpaceSourceEvidenceEventId,
"address_space_source_evidence_event_id",
{ json_event_id_option(binding.address_space_source_evidence_event_id);
}),
(MemoryMapSourceEvidenceEventId,
"memory_map_source_evidence_event_id",
{ json_event_id_option(binding.memory_map_source_evidence_event_id);
}),
(CapabilityImportTableSourceEvidenceEventId,
"capability_import_table_source_evidence_event_id",
{ json_event_id_option(binding.capability_import_table_source_evidence_event_id);
}),
(AuditRollbackWriteBoundarySourceEvidenceEventId,
"audit_rollback_write_boundary_source_evidence_event_id",
{ json_event_id_option(binding.audit_rollback_write_boundary_source_evidence_event_id);
}),
(RetainedModuleEvidenceEventIds,
"retained_module_evidence_event_ids",
{ raw("{\"manifest_reference_event_id\": ");
json_event_id_option(binding.manifest_reference_event_id);
raw(", \"candidate_artifact_reference_event_id\": ");
json_event_id_option(binding.artifact_reference_event_id);
raw(", \"vm_test_report_reference_event_id\": ");
json_event_id_option(binding.vm_test_report_reference_event_id);
raw(", \"local_attestation_reference_event_id\": ");
json_event_id_option(binding.local_attestation_reference_event_id);
raw(", \"local_approval_reference_event_id\": ");
json_event_id_option(binding.local_approval_reference_event_id);
raw(", \"computed_grant_reference_event_id\": ");
json_event_id_option(binding.computed_grant_reference_event_id);
raw(", \"audit_rollback_reference_event_id\": ");
json_event_id_option(binding.audit_rollback_reference_event_id);
raw(", \"service_slot_reservation_event_id\": ");
json_event_id_option(binding.service_slot_reservation_event_id);
raw("}");
}),
(RamOnlyServiceSlotId,
"ram_only_service_slot_id",
{ if let Some(id) = binding.ram_only_service_slot_id { json_str(id.as_str());
} else { raw("null");
} }),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(AcceptsLoaderDescriptor,
"accepts_loader_descriptor",
{ raw_bool(binding.accepts_loader_descriptor);
}),
(AcceptsDescriptorBytes,
"accepts_descriptor_bytes",
{ raw_bool(binding.accepts_descriptor_bytes);
}),
(ProducesParsedDescriptor,
"produces_parsed_descriptor",
{ raw("false");
}),
(ValidatesDescriptorSchema,
"validates_descriptor_schema",
{ raw("false");
}),
(ProducesValidatedDescriptor,
"produces_validated_descriptor",
{ raw("false");
}),
(ValidatesDescriptorCapabilities,
"validates_descriptor_capabilities",
{ raw("false");
}),
(ProducesCapabilityValidatedDescriptor,
"produces_capability_validated_descriptor",
{ raw("false");
}),
(AuthorizesExecutableLoadPlan,
"authorizes_executable_load_plan",
{ raw("false");
}),
(ProducesExecutableLoadPlan,
"produces_executable_load_plan",
{ raw("false");
}),
(ProducesExecutableImageLayout,
"produces_executable_image_layout",
{ raw("false");
}),
(ProducesExecutablePageMappingPlan,
"produces_executable_page_mapping_plan",
{ raw("false");
}),
(BindsCapabilityValidatedDescriptorToExecutablePages,
"binds_capability_validated_descriptor_to_executable_pages",
{ raw("false");
}),
(ParsesDescriptorBytes,
"parses_descriptor_bytes",
{ raw("false");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw_bool(binding.accepts_artifact_bytes);
}),
(AuthorizesDescriptorIntake,
"authorizes_descriptor_intake",
{ raw_bool(binding.authorizes_descriptor_intake);
}),
(AuthorizesArtifactByteIntake,
"authorizes_artifact_byte_intake",
{ raw_bool(binding.authorizes_artifact_byte_intake);
}),
(MapsExecutablePages,
"maps_executable_pages",
{ raw_bool(binding.maps_executable_pages);
}),
(JumpsToEntrypoint,
"jumps_to_entrypoint",
{ raw_bool(binding.jumps_to_entrypoint);
}),
(AuthorizesExecution,
"authorizes_execution",
{ raw_bool(binding.authorizes_execution);
}),
(MutatesServiceRegistry,
"mutates_service_registry",
{ raw_bool(binding.mutates_service_registry);
}),
(WritesDurableAuditState,
"writes_durable_audit_state",
{ raw_bool(binding.writes_durable_audit_state);
}),
(InstallsRollbackState,
"installs_rollback_state",
{ raw_bool(binding.installs_rollback_state);
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw_bool(binding.allocates_service_slot);
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw_bool(binding.creates_service_inventory_records);
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadsArtifact,
"loads_artifact",
{ raw_bool(binding.loads_artifact);
}),
(StartsService,
"starts_service",
{ raw_bool(binding.starts_service);
}),
(MarksServiceRunning,
"marks_service_running",
{ raw_bool(binding.marks_service_running);
}),
(CreatesServiceHealthRecords,
"creates_service_health_records",
{ raw_bool(binding.creates_service_health_records);
}),
(WritesServiceStartAuditRecord,
"writes_service_start_audit_record",
{ raw_bool(binding.writes_service_start_audit_record);
}),
(UnloadsService,
"unloads_service",
{ raw_bool(binding.unloads_service);
}),
(CleansUpServiceSlot,
"cleans_up_service_slot",
{ raw_bool(binding.cleans_up_service_slot);
}),
(CommitsLiveLoad,
"commits_live_load",
{ raw_bool(binding.commits_live_load);
}),
(WritesLoadCommitAuditRecord,
"writes_load_commit_audit_record",
{ raw_bool(binding.writes_load_commit_audit_record);
}),
(InstallsCommitRollbackRecord,
"installs_commit_rollback_record",
{ raw_bool(binding.installs_commit_rollback_record);
}),
(RecordsLoadResult,
"records_load_result",
{ raw_bool(binding.records_load_result);
}),
(LoadAttempted,
"load_attempted",
{ raw_bool(binding.load_attempted);
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
}

fn emit_module_loader_live_load_boundary_binding(
    kind: &str,
    binding: &event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_LOADER_LIVE_LOAD_BOUNDARY_BINDING_FIELDS,
        emit_module_loader_live_load_boundary_binding_value,
    );
}

define_direct_binding_fields! { HelloRecoveryRollbackInspectSourceReferenceBindingField,
HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_BINDING_FIELDS,
emit_hello_recovery_rollback_inspect_source_reference_binding_value,
event_log::HelloRecoveryRollbackInspectSourceReferenceBinding,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_rollback_inspect_source_reference_binding.v0\"");
}),
(Status,
"status",
{ raw("\"retained_audit_event_verified\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(SourceEventId,
"source_event_id",
{ json_event_id(binding.source_event_id);
}),
(ReferenceHash,
"reference_hash",
{ json_sha256(binding.reference_hash);
}),
(InspectionHash,
"inspection_hash",
{ json_sha256(binding.inspection_hash);
}),
(SourceSectorPlanHash,
"source_sector_plan_hash",
{ json_sha256(binding.source_sector_plan_hash);
}),
(SourceTargetRegionWriteReadbackHash,
"source_target_region_write_readback_hash",
{ json_sha256(binding.source_target_region_write_readback_hash);
}),
(AuthorizesRollbackApply,
"authorizes_rollback_apply",
{ raw_bool(binding.authorizes_rollback_apply);
}),
}

fn emit_hello_recovery_rollback_inspect_source_reference_binding(
    kind: &str,
    binding: &event_log::HelloRecoveryRollbackInspectSourceReferenceBinding,
) {
    emit_binding_object_direct(
        binding,
        kind,
        HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_BINDING_FIELDS,
        emit_hello_recovery_rollback_inspect_source_reference_binding_value,
    );
}

define_direct_binding_fields! { AgentCommandEnvelopeDecisionBindingField,
AGENT_COMMAND_ENVELOPE_DECISION_BINDING_FIELDS,
emit_agent_command_envelope_decision_binding_value,
event_log::AgentCommandEnvelopeBinding,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.agent_command_envelope.audit_binding.v0\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(CommandSchema,
"command_schema",
{ raw("\"raios.agent_command_envelope.v0\"");
}),
(SchemaOk,
"schema_ok",
{ raw_bool(binding.schema_ok);
}),
(TargetMethod,
"target_method",
{ json_opt_str(binding.target_method);
}),
(TargetMethodAllowed,
"target_method_allowed",
{ raw_bool(binding.target_method_allowed);
}),
(RequestedCapability,
"requested_capability",
{ json_opt_str(binding.requested_capability);
}),
(RequestedCapabilityAllowed,
"requested_capability_allowed",
{ raw_bool(binding.requested_capability_allowed);
}),
(SubmittedClassification,
"submitted_classification",
{ json_opt_str(binding.submitted_classification);
}),
(ClassificationAllowed,
"classification_allowed",
{ raw_bool(binding.classification_allowed);
}),
(Accepted,
"accepted",
{ raw_bool(binding.accepted);
}),
(Code,
"code",
{ json_str(binding.code);
}),
(Reason,
"reason",
{ json_str(binding.reason);
}),
(DispatchesExistingAgentMethod,
"dispatches_existing_agent_method",
{ raw_bool(binding.dispatches_existing_agent_method);
}),
(CreatesParallelDispatcher,
"creates_parallel_dispatcher",
{ raw_bool(binding.creates_parallel_dispatcher);
}),
(ProviderWrite,
"provider_write",
{ json_str(binding.provider_write);
}),
(LoadsCandidateBytes,
"loads_candidate_bytes",
{ raw_bool(binding.loads_candidate_bytes);
}),
(WritesPersistentState,
"writes_persistent_state",
{ raw_bool(binding.writes_persistent_state);
}),
(WritesDurableAuditLog,
"writes_durable_audit_log",
{ raw_bool(binding.writes_durable_audit_log);
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw_bool(binding.installs_rollback_plan);
}),
(GrantsBroadMutation,
"grants_broad_mutation",
{ raw_bool(binding.grants_broad_mutation);
}),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
}

fn emit_agent_command_envelope_decision_binding(
    kind: &str,
    binding: &event_log::AgentCommandEnvelopeBinding,
) {
    emit_binding_object_direct(
        binding,
        kind,
        AGENT_COMMAND_ENVELOPE_DECISION_BINDING_FIELDS,
        emit_agent_command_envelope_decision_binding_value,
    );
}

define_direct_binding_fields! { ProviderRequestEnvelopeBindingField,
PROVIDER_REQUEST_ENVELOPE_BINDING_FIELDS,
emit_provider_request_envelope_binding_value,
event_log::ProviderRequestEnvelopeBinding,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.provider_request_envelope.v0\"");
}),
(Status,
"status",
{ raw("\"local_prewrite_envelope\"");
}),
(SatisfiesCurrentBootExportGate,
"satisfies_current_boot_export_gate",
{ raw("false");
}),
(ProviderWrite,
"provider_write",
{ raw("\"not_attempted\"");
}),
(ContextAttachedToProviderBody,
"context_attached_to_provider_body",
{ raw("false");
}),
(RequestId,
"request_id",
{ raw_fmt(format_args!("{}",
binding.request_id));
}),
(RequestBodyHash,
"request_body_hash",
{ json_sha256(binding.request_body_hash);
}),
(EnvelopeHash,
"envelope_hash",
{ json_sha256(binding.envelope_hash);
}),
(TrustSnapshot,
"trust_snapshot",
{ raw("{\"provider_trust_state\": ");
json_str(binding.provider_trust_state);
raw(", \"provider_trust_positive\": ");
raw_bool(binding.provider_trust_positive);
raw(", \"development_tls_bypass\": ");
raw_bool(binding.development_tls_bypass);
raw("}");
}),
}

fn emit_provider_request_envelope_binding(
    kind: &str,
    binding: &event_log::ProviderRequestEnvelopeBinding,
) {
    emit_binding_object_direct(
        binding,
        kind,
        PROVIDER_REQUEST_ENVELOPE_BINDING_FIELDS,
        emit_provider_request_envelope_binding_value,
    );
}

define_direct_binding_fields! { ProviderRequestBoundBindingField,
PROVIDER_REQUEST_BOUND_BINDING_FIELDS,
emit_provider_request_bound_binding_value,
event_log::ProviderRequestBinding,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.provider_request_binding.v0\"");
}),
(Status,
"status",
{ raw("\"bound\"");
}),
(SatisfiesRequestBindingGate,
"satisfies_request_binding_gate",
{ raw("true");
}),
(SatisfiesCurrentBootExportGate,
"satisfies_current_boot_export_gate",
{ raw("false");
}),
(ProviderWriteAtBinding,
"provider_write_at_binding",
{ raw("\"not_attempted\"");
}),
(ContextAttachedToProviderBody,
"context_attached_to_provider_body",
{ raw("false");
}),
(RequestId,
"request_id",
{ raw_fmt(format_args!("{}",
binding.request_id));
}),
(RequestEnvelopeEventId,
"request_envelope_event_id",
{ json_event_id(binding.request_envelope_event_id);
}),
(RequestBodyHash,
"request_body_hash",
{ json_sha256(binding.request_body_hash);
}),
(RequestEnvelopeHash,
"request_envelope_hash",
{ json_sha256(binding.request_envelope_hash);
}),
(RequestBindingHash,
"request_binding_hash",
{ json_sha256(binding.request_binding_hash);
}),
(TrustSnapshot,
"trust_snapshot",
{ raw("{\"provider_trust_state\": ");
json_str(binding.provider_trust_state);
raw(", \"provider_trust_positive\": true, \"provider_trust_pin_kind\": ");
json_opt_str(binding.provider_trust_pin_kind);
raw(", \"provider_trust_pin_id\": ");
json_opt_str(binding.provider_trust_pin_id);
raw(", \"provider_trust_pin_slot\": ");
json_opt_str(binding.provider_trust_pin_slot);
raw(", \"provider_trust_pin_rotation_policy\": ");
json_str(binding.provider_trust_pin_rotation_policy);
raw(", \"provider_trust_pin_rotation_id\": ");
json_opt_str(binding.provider_trust_pin_rotation_id);
raw(", \"provider_trust_verifier\": ");
emit_provider_trust_verifier_metadata(binding.provider_trust_verifier);
raw(", \"provider_trust_verifier_decision\": ");
emit_provider_trust_verifier_decision(binding.provider_trust_verifier_decision);
raw(", \"provider_trust_evidence_hash\": ");
json_sha256(binding.provider_trust_evidence_hash);
raw(", \"development_tls_bypass\": ");
raw_bool(binding.development_tls_bypass);
raw("}");
}),
(Hashes,
"hashes",
{ emit_provider_context_hashes(binding.context);
}),
}

fn emit_provider_request_bound_binding(kind: &str, binding: &event_log::ProviderRequestBinding) {
    emit_binding_object_direct(
        binding,
        kind,
        PROVIDER_REQUEST_BOUND_BINDING_FIELDS,
        emit_provider_request_bound_binding_value,
    );
}

define_direct_binding_fields! { ProviderExportAuditBoundBindingField,
PROVIDER_EXPORT_AUDIT_BOUND_BINDING_FIELDS,
emit_provider_export_audit_bound_binding_value,
event_log::ProviderExportAuditBinding,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.provider_context_export_audit_binding.v0\"");
}),
(Status,
"status",
{ raw("\"authorized_for_single_provider_request\"");
}),
(SatisfiesExportAuditBindingGate,
"satisfies_export_audit_binding_gate",
{ raw("true");
}),
(SatisfiesCurrentBootExportGate,
"satisfies_current_boot_export_gate",
{ raw("false");
}),
(PositiveExportAuthorization,
"positive_export_authorization",
{ raw("true");
}),
(AutomaticContextInjection,
"automatic_context_injection",
{ raw("\"disabled\"");
}),
(ProviderWriteAtBinding,
"provider_write_at_binding",
{ raw("\"not_attempted\"");
}),
(ContextAttachedToProviderBody,
"context_attached_to_provider_body",
{ raw_bool(binding.context_attached_to_provider_body);
}),
(RequestId,
"request_id",
{ raw_fmt(format_args!("{}",
binding.request_id));
}),
(RequestEnvelopeEventId,
"request_envelope_event_id",
{ json_event_id(binding.request_envelope_event_id);
}),
(RequestBindingEventId,
"request_binding_event_id",
{ json_event_id(binding.request_binding_event_id);
}),
(RequestBodyHash,
"request_body_hash",
{ json_sha256(binding.request_body_hash);
}),
(RequestEnvelopeHash,
"request_envelope_hash",
{ json_sha256(binding.request_envelope_hash);
}),
(RequestBindingHash,
"request_binding_hash",
{ json_sha256(binding.request_binding_hash);
}),
(ExportAuditBindingHash,
"export_audit_binding_hash",
{ json_sha256(binding.export_audit_binding_hash);
}),
(TrustSnapshot,
"trust_snapshot",
{ raw("{\"provider_trust_state\": ");
json_str(binding.provider_trust_state);
raw(", \"provider_trust_positive\": true, \"provider_trust_pin_kind\": ");
json_opt_str(binding.provider_trust_pin_kind);
raw(", \"provider_trust_pin_id\": ");
json_opt_str(binding.provider_trust_pin_id);
raw(", \"provider_trust_pin_slot\": ");
json_opt_str(binding.provider_trust_pin_slot);
raw(", \"provider_trust_pin_rotation_policy\": ");
json_str(binding.provider_trust_pin_rotation_policy);
raw(", \"provider_trust_pin_rotation_id\": ");
json_opt_str(binding.provider_trust_pin_rotation_id);
raw(", \"provider_trust_verifier\": ");
emit_provider_trust_verifier_metadata(binding.provider_trust_verifier);
raw(", \"provider_trust_verifier_decision\": ");
emit_provider_trust_verifier_decision(binding.provider_trust_verifier_decision);
raw(", \"provider_trust_evidence_hash\": ");
json_sha256(binding.provider_trust_evidence_hash);
raw(", \"development_tls_bypass\": false}");
}),
(Hashes,
"hashes",
{ emit_provider_context_hashes(binding.context);
}),
}

fn emit_provider_export_audit_bound_binding(
    kind: &str,
    binding: &event_log::ProviderExportAuditBinding,
) {
    emit_binding_object_direct(
        binding,
        kind,
        PROVIDER_EXPORT_AUDIT_BOUND_BINDING_FIELDS,
        emit_provider_export_audit_bound_binding_value,
    );
}

define_direct_binding_fields! { ProviderBindingConsumptionBindingField,
PROVIDER_BINDING_CONSUMPTION_BINDING_FIELDS,
emit_provider_binding_consumption_binding_value,
event_log::ProviderBindingConsumption,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.provider_context_binding_consumption.v0\"");
}),
(Status,
"status",
{ raw("\"consumed_for_gate_evaluation\"");
}),
(SatisfiesCurrentBootExportGate,
"satisfies_current_boot_export_gate",
{ raw("false");
}),
(AutomaticContextInjection,
"automatic_context_injection",
{ raw("\"disabled\"");
}),
(ProviderWrite,
"provider_write",
{ raw("\"not_attempted\"");
}),
(ContextAttachedToProviderBody,
"context_attached_to_provider_body",
{ raw("false");
}),
(RequestId,
"request_id",
{ raw_fmt(format_args!("{}",
binding.request_id));
}),
(RequestEnvelopeEventId,
"request_envelope_event_id",
{ json_event_id(binding.request_envelope_event_id);
}),
(RequestBindingEventId,
"request_binding_event_id",
{ json_event_id(binding.request_binding_event_id);
}),
(ExportAuditBindingEventId,
"export_audit_binding_event_id",
{ json_event_id(binding.export_audit_binding_event_id);
}),
(RequestBindingHash,
"request_binding_hash",
{ json_sha256(binding.request_binding_hash);
}),
(ExportAuditBindingHash,
"export_audit_binding_hash",
{ json_sha256(binding.export_audit_binding_hash);
}),
(ProviderTrustEvidenceHash,
"provider_trust_evidence_hash",
{ json_sha256(binding.provider_trust_evidence_hash);
}),
(Hashes,
"hashes",
{ emit_provider_context_hashes(binding.context);
}),
}

fn emit_provider_binding_consumption_binding(
    kind: &str,
    binding: &event_log::ProviderBindingConsumption,
) {
    emit_binding_object_direct(
        binding,
        kind,
        PROVIDER_BINDING_CONSUMPTION_BINDING_FIELDS,
        emit_provider_binding_consumption_binding_value,
    );
}

define_direct_binding_fields! { ProviderContextInjectionAuthorizationBindingField,
PROVIDER_CONTEXT_INJECTION_AUTHORIZATION_BINDING_FIELDS,
emit_provider_context_injection_authorization_binding_value,
event_log::ProviderContextInjectionAuthorization,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.provider_context_injection_authorization.v0\"");
}),
(Status,
"status",
{ raw("\"authorized_for_single_provider_request\"");
}),
(SatisfiesCurrentBootExportGate,
"satisfies_current_boot_export_gate",
{ raw("false");
}),
(AutomaticContextInjection,
"automatic_context_injection",
{ raw("\"disabled\"");
}),
(ProviderWrite,
"provider_write",
{ raw("\"not_attempted\"");
}),
(ContextAttachedToProviderBody,
"context_attached_to_provider_body",
{ raw_bool(binding.context_attached_to_provider_body);
}),
(RequestId,
"request_id",
{ raw_fmt(format_args!("{}",
binding.request_id));
}),
(RequestEnvelopeEventId,
"request_envelope_event_id",
{ json_event_id(binding.request_envelope_event_id);
}),
(RequestBindingEventId,
"request_binding_event_id",
{ json_event_id(binding.request_binding_event_id);
}),
(ExportAuditBindingEventId,
"export_audit_binding_event_id",
{ json_event_id(binding.export_audit_binding_event_id);
}),
(BindingConsumptionEventId,
"binding_consumption_event_id",
{ json_event_id(binding.binding_consumption_event_id);
}),
(RequestBodyHash,
"request_body_hash",
{ json_sha256(binding.request_body_hash);
}),
(RequestEnvelopeHash,
"request_envelope_hash",
{ json_sha256(binding.request_envelope_hash);
}),
(RequestBindingHash,
"request_binding_hash",
{ json_sha256(binding.request_binding_hash);
}),
(ExportAuditBindingHash,
"export_audit_binding_hash",
{ json_sha256(binding.export_audit_binding_hash);
}),
(FinalAuthorizationHash,
"final_authorization_hash",
{ json_sha256(binding.final_authorization_hash);
}),
(TrustSnapshot,
"trust_snapshot",
{ raw("{\"provider_trust_state\": ");
json_str(binding.provider_trust_state);
raw(", \"provider_trust_positive\": true, \"provider_trust_evidence_hash\": ");
json_sha256(binding.provider_trust_evidence_hash);
raw("}");
}),
(Hashes,
"hashes",
{ emit_provider_context_hashes(binding.context);
}),
}

fn emit_provider_context_injection_authorization_binding(
    kind: &str,
    binding: &event_log::ProviderContextInjectionAuthorization,
) {
    emit_binding_object_direct(
        binding,
        kind,
        PROVIDER_CONTEXT_INJECTION_AUTHORIZATION_BINDING_FIELDS,
        emit_provider_context_injection_authorization_binding_value,
    );
}

define_direct_binding_fields! { ProviderRequestBindingDeniedBindingField,
PROVIDER_REQUEST_BINDING_DENIED_BINDING_FIELDS,
emit_provider_request_binding_denied_binding_value,
event_log::ProviderContextHashes,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.provider_request_binding_denial.v0\"");
}),
(Status,
"status",
{ raw("\"denied_not_bound\"");
}),
(SatisfiesCurrentBootExportGate,
"satisfies_current_boot_export_gate",
{ raw("false");
}),
(ProviderWrite,
"provider_write",
{ raw("\"not_attempted\"");
}),
(Hashes,
"hashes",
{ emit_provider_context_hashes(*binding);
}),
}

fn emit_provider_request_binding_denied_binding(
    kind: &str,
    binding: &event_log::ProviderContextHashes,
) {
    emit_binding_object_direct(
        binding,
        kind,
        PROVIDER_REQUEST_BINDING_DENIED_BINDING_FIELDS,
        emit_provider_request_binding_denied_binding_value,
    );
}

define_direct_binding_fields! { ProviderExportDenialAuditBindingField,
PROVIDER_EXPORT_DENIAL_AUDIT_BINDING_FIELDS,
emit_provider_export_denial_audit_binding_value,
event_log::ProviderContextHashes,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.provider_context_export_denial_audit.v0\"");
}),
(Status,
"status",
{ raw("\"denied_no_provider_write\"");
}),
(SatisfiesCurrentBootExportGate,
"satisfies_current_boot_export_gate",
{ raw("false");
}),
(PositiveExportAuthorization,
"positive_export_authorization",
{ raw("false");
}),
(ProviderWrite,
"provider_write",
{ raw("\"not_attempted\"");
}),
(Hashes,
"hashes",
{ emit_provider_context_hashes(*binding);
}),
}

fn emit_provider_export_denial_audit_binding(
    kind: &str,
    binding: &event_log::ProviderContextHashes,
) {
    emit_binding_object_direct(
        binding,
        kind,
        PROVIDER_EXPORT_DENIAL_AUDIT_BINDING_FIELDS,
        emit_provider_export_denial_audit_binding_value,
    );
}

define_direct_binding_fields! { ModuleManifestReferenceBindingField,
MODULE_MANIFEST_REFERENCE_BINDING_FIELDS,
emit_module_manifest_reference_binding_value,
event_log::ModuleManifestReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.module_manifest_reference.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_load_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.module.load_ephemeral\"");
}),
(LoadMode,
"load_mode",
{ raw("\"ram_only\"");
}),
(ManifestSchema,
"manifest_schema",
{ raw("\"raios.module_manifest.v0\"");
}),
(AcceptsManifestJson,
"accepts_manifest_json",
{ raw("false");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw("false");
}),
(AcceptsUnsignedServiceCode,
"accepts_unsigned_service_code",
{ raw("false");
}),
(AuthorizesGuestLoad,
"authorizes_guest_load",
{ raw("false");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(Hashes,
"hashes",
{ raw("{\"manifest_reference_hash\": ");
json_sha256(binding.manifest_reference_hash);
raw(", \"manifest_hash\": ");
json_sha256(binding.manifest_hash);
raw("}");
}),
}

fn emit_module_manifest_reference_binding(
    kind: &str,
    binding: &event_log::ModuleManifestReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_MANIFEST_REFERENCE_BINDING_FIELDS,
        emit_module_manifest_reference_binding_value,
    );
}

define_direct_binding_fields! { ModuleCandidateArtifactReferenceBindingField,
MODULE_CANDIDATE_ARTIFACT_REFERENCE_BINDING_FIELDS,
emit_module_candidate_artifact_reference_binding_value,
event_log::ModuleCandidateArtifactReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.module_candidate_artifact_reference.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_load_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.module.load_ephemeral\"");
}),
(LoadMode,
"load_mode",
{ raw("\"ram_only\"");
}),
(AcceptsManifestJson,
"accepts_manifest_json",
{ raw("false");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw("false");
}),
(AcceptsUnsignedServiceCode,
"accepts_unsigned_service_code",
{ raw("false");
}),
(AuthorizesGuestLoad,
"authorizes_guest_load",
{ raw("false");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedManifestReferenceEventId,
"retained_manifest_reference_event_id",
{ json_event_id(binding.retained_manifest_reference_event_id);
}),
(RetainedComputedGrantReferenceEventId,
"retained_computed_grant_reference_event_id",
{ json_event_id(binding.retained_reference_event_id);
}),
(Hashes,
"hashes",
{ raw("{\"artifact_reference_hash\": ");
json_sha256(binding.artifact_reference_hash);
raw(", \"manifest_reference_hash\": ");
json_sha256(binding.manifest_reference_hash);
raw(", \"manifest_hash\": ");
json_sha256(binding.manifest_hash);
raw(", \"computed_capability_grant_hash\": ");
json_sha256(binding.computed_grant_hash);
raw(", \"artifact_hash\": ");
json_sha256(binding.artifact_hash);
raw(", \"vm_test_report_hash\": ");
json_sha256(binding.vm_report_hash);
raw(", \"local_attestation_hash\": ");
json_sha256(binding.local_attestation_hash);
raw("}");
}),
}

fn emit_module_candidate_artifact_reference_binding(
    kind: &str,
    binding: &event_log::ModuleCandidateArtifactReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_CANDIDATE_ARTIFACT_REFERENCE_BINDING_FIELDS,
        emit_module_candidate_artifact_reference_binding_value,
    );
}

define_direct_binding_fields! { ModuleVmTestReportReferenceBindingField,
MODULE_VM_TEST_REPORT_REFERENCE_BINDING_FIELDS,
emit_module_vm_test_report_reference_binding_value,
event_log::ModuleVmTestReportReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.module_vm_test_report_reference.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_load_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.module.load_ephemeral\"");
}),
(LoadMode,
"load_mode",
{ raw("\"ram_only\"");
}),
(VmTestReportSchema,
"vm_test_report_schema",
{ raw("\"raios.vm_test_report.v0\"");
}),
(AcceptsManifestJson,
"accepts_manifest_json",
{ raw("false");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw("false");
}),
(AcceptsVmReportJson,
"accepts_vm_report_json",
{ raw("false");
}),
(AcceptsUnsignedServiceCode,
"accepts_unsigned_service_code",
{ raw("false");
}),
(AuthorizesGuestLoad,
"authorizes_guest_load",
{ raw("false");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedManifestReferenceEventId,
"retained_manifest_reference_event_id",
{ json_event_id(binding.retained_manifest_reference_event_id);
}),
(RetainedCandidateArtifactReferenceEventId,
"retained_candidate_artifact_reference_event_id",
{ json_event_id(binding.retained_artifact_reference_event_id);
}),
(RetainedComputedGrantReferenceEventId,
"retained_computed_grant_reference_event_id",
{ json_event_id(binding.retained_reference_event_id);
}),
(Hashes,
"hashes",
{ raw("{\"vm_test_report_reference_hash\": ");
json_sha256(binding.report_reference_hash);
raw(", \"manifest_reference_hash\": ");
json_sha256(binding.manifest_reference_hash);
raw(", \"artifact_reference_hash\": ");
json_sha256(binding.artifact_reference_hash);
raw(", \"manifest_hash\": ");
json_sha256(binding.manifest_hash);
raw(", \"artifact_hash\": ");
json_sha256(binding.artifact_hash);
raw(", \"computed_capability_grant_hash\": ");
json_sha256(binding.computed_grant_hash);
raw(", \"vm_test_report_hash\": ");
json_sha256(binding.vm_report_hash);
raw(", \"local_attestation_hash\": ");
json_sha256(binding.local_attestation_hash);
raw("}");
}),
}

fn emit_module_vm_test_report_reference_binding(
    kind: &str,
    binding: &event_log::ModuleVmTestReportReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_VM_TEST_REPORT_REFERENCE_BINDING_FIELDS,
        emit_module_vm_test_report_reference_binding_value,
    );
}

define_direct_binding_fields! { ModuleLocalAttestationReferenceBindingField,
MODULE_LOCAL_ATTESTATION_REFERENCE_BINDING_FIELDS,
emit_module_local_attestation_reference_binding_value,
event_log::ModuleLocalAttestationReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.module_local_attestation_reference.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_load_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.module.load_ephemeral\"");
}),
(LoadMode,
"load_mode",
{ raw("\"ram_only\"");
}),
(LocalAttestationSchema,
"local_attestation_schema",
{ raw("\"raios.local_attestation.v0\"");
}),
(AcceptsLocalAttestationJson,
"accepts_local_attestation_json",
{ raw("false");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw("false");
}),
(AcceptsUnsignedServiceCode,
"accepts_unsigned_service_code",
{ raw("false");
}),
(AuthorizesGuestLoad,
"authorizes_guest_load",
{ raw("false");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedManifestReferenceEventId,
"retained_manifest_reference_event_id",
{ json_event_id(binding.retained_manifest_reference_event_id);
}),
(RetainedCandidateArtifactReferenceEventId,
"retained_candidate_artifact_reference_event_id",
{ json_event_id(binding.retained_artifact_reference_event_id);
}),
(RetainedVmTestReportReferenceEventId,
"retained_vm_test_report_reference_event_id",
{ json_event_id(binding.retained_vm_report_reference_event_id);
}),
(RetainedComputedGrantReferenceEventId,
"retained_computed_grant_reference_event_id",
{ json_event_id(binding.retained_reference_event_id);
}),
(Hashes,
"hashes",
{ raw("{\"local_attestation_reference_hash\": ");
json_sha256(binding.attestation_reference_hash);
raw(", \"manifest_reference_hash\": ");
json_sha256(binding.manifest_reference_hash);
raw(", \"artifact_reference_hash\": ");
json_sha256(binding.artifact_reference_hash);
raw(", \"vm_test_report_reference_hash\": ");
json_sha256(binding.vm_report_reference_hash);
raw(", \"manifest_hash\": ");
json_sha256(binding.manifest_hash);
raw(", \"artifact_hash\": ");
json_sha256(binding.artifact_hash);
raw(", \"computed_capability_grant_hash\": ");
json_sha256(binding.computed_grant_hash);
raw(", \"vm_test_report_hash\": ");
json_sha256(binding.vm_report_hash);
raw(", \"local_attestation_hash\": ");
json_sha256(binding.local_attestation_hash);
raw("}");
}),
}

fn emit_module_local_attestation_reference_binding(
    kind: &str,
    binding: &event_log::ModuleLocalAttestationReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_LOCAL_ATTESTATION_REFERENCE_BINDING_FIELDS,
        emit_module_local_attestation_reference_binding_value,
    );
}

define_direct_binding_fields! { ModuleLocalApprovalReferenceBindingField,
MODULE_LOCAL_APPROVAL_REFERENCE_BINDING_FIELDS,
emit_module_local_approval_reference_binding_value,
event_log::ModuleLocalApprovalReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.module_local_approval_reference.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_load_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.module.load_ephemeral\"");
}),
(LoadMode,
"load_mode",
{ raw("\"ram_only\"");
}),
(LocalApprovalSchema,
"local_approval_schema",
{ raw("\"raios.local_approval.v0\"");
}),
(AcceptsLocalApprovalText,
"accepts_local_approval_text",
{ raw("false");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw("false");
}),
(AcceptsUnsignedServiceCode,
"accepts_unsigned_service_code",
{ raw("false");
}),
(AuthorizesGuestLoad,
"authorizes_guest_load",
{ raw("false");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedManifestReferenceEventId,
"retained_manifest_reference_event_id",
{ json_event_id(binding.retained_manifest_reference_event_id);
}),
(RetainedCandidateArtifactReferenceEventId,
"retained_candidate_artifact_reference_event_id",
{ json_event_id(binding.retained_artifact_reference_event_id);
}),
(RetainedVmTestReportReferenceEventId,
"retained_vm_test_report_reference_event_id",
{ json_event_id(binding.retained_vm_report_reference_event_id);
}),
(RetainedLocalAttestationReferenceEventId,
"retained_local_attestation_reference_event_id",
{ json_event_id(binding.retained_local_attestation_reference_event_id);
}),
(RetainedComputedGrantReferenceEventId,
"retained_computed_grant_reference_event_id",
{ json_event_id(binding.retained_reference_event_id);
}),
(Hashes,
"hashes",
{ raw("{\"local_approval_reference_hash\": ");
json_sha256(binding.approval_reference_hash);
raw(", \"manifest_reference_hash\": ");
json_sha256(binding.manifest_reference_hash);
raw(", \"artifact_reference_hash\": ");
json_sha256(binding.artifact_reference_hash);
raw(", \"vm_test_report_reference_hash\": ");
json_sha256(binding.vm_report_reference_hash);
raw(", \"local_attestation_reference_hash\": ");
json_sha256(binding.local_attestation_reference_hash);
raw(", \"manifest_hash\": ");
json_sha256(binding.manifest_hash);
raw(", \"artifact_hash\": ");
json_sha256(binding.artifact_hash);
raw(", \"computed_capability_grant_hash\": ");
json_sha256(binding.computed_grant_hash);
raw(", \"vm_test_report_hash\": ");
json_sha256(binding.vm_report_hash);
raw(", \"local_attestation_hash\": ");
json_sha256(binding.local_attestation_hash);
raw(", \"local_approval_hash\": ");
json_sha256(binding.local_approval_hash);
raw("}");
}),
}

fn emit_module_local_approval_reference_binding(
    kind: &str,
    binding: &event_log::ModuleLocalApprovalReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_LOCAL_APPROVAL_REFERENCE_BINDING_FIELDS,
        emit_module_local_approval_reference_binding_value,
    );
}

define_direct_binding_fields! { ModuleComputedGrantReferenceBindingField,
MODULE_COMPUTED_GRANT_REFERENCE_BINDING_FIELDS,
emit_module_computed_grant_reference_binding_value,
event_log::ModuleComputedGrantReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.module_computed_grant_reference.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_load_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.module.load_ephemeral\"");
}),
(LoadMode,
"load_mode",
{ raw("\"ram_only\"");
}),
(GrantsCapability,
"grants_capability",
{ raw("false");
}),
(GrantsLoadNow,
"grants_load_now",
{ raw("false");
}),
(AuthorizesGuestLoad,
"authorizes_guest_load",
{ raw("false");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(Hashes,
"hashes",
{ raw("{\"computed_capability_grant_hash\": ");
json_sha256(binding.computed_grant_hash);
raw(", \"manifest_hash\": ");
json_sha256(binding.manifest_hash);
raw(", \"artifact_hash\": ");
json_sha256(binding.artifact_hash);
raw(", \"vm_test_report_hash\": ");
json_sha256(binding.vm_report_hash);
raw(", \"local_attestation_hash\": ");
json_sha256(binding.local_attestation_hash);
raw("}");
}),
}

fn emit_module_computed_grant_reference_binding(
    kind: &str,
    binding: &event_log::ModuleComputedGrantReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_COMPUTED_GRANT_REFERENCE_BINDING_FIELDS,
        emit_module_computed_grant_reference_binding_value,
    );
}

define_direct_binding_fields! { ModuleAuditRollbackReferenceBindingField,
MODULE_AUDIT_ROLLBACK_REFERENCE_BINDING_FIELDS,
emit_module_audit_rollback_reference_binding_value,
event_log::ModuleAuditRollbackReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.module_audit_rollback_reference.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_load_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.module.load_ephemeral\"");
}),
(LoadMode,
"load_mode",
{ raw("\"ram_only\"");
}),
(DurableAuditWritten,
"durable_audit_written",
{ raw("false");
}),
(RollbackPlanInstalled,
"rollback_plan_installed",
{ raw("false");
}),
(GrantsCapability,
"grants_capability",
{ raw("false");
}),
(GrantsLoadNow,
"grants_load_now",
{ raw("false");
}),
(AuthorizesGuestLoad,
"authorizes_guest_load",
{ raw("false");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(DenialEventId,
"denial_event_id",
{ json_event_id(binding.denial_event_id);
}),
(RetainedComputedGrantReferenceEventId,
"retained_computed_grant_reference_event_id",
{ json_event_id(binding.retained_reference_event_id);
}),
(RamOnlyServiceSlotId,
"ram_only_service_slot_id",
{ json_str(binding.ram_only_service_slot_id.as_str());
}),
(Hashes,
"hashes",
{ raw("{\"audit_record_hash\": ");
json_sha256(binding.audit_record_hash);
raw(", \"rollback_plan_hash\": ");
json_sha256(binding.rollback_plan_hash);
raw(", \"computed_capability_grant_hash\": ");
json_sha256(binding.computed_grant_hash);
raw(", \"manifest_hash\": ");
json_sha256(binding.manifest_hash);
raw(", \"artifact_hash\": ");
json_sha256(binding.artifact_hash);
raw(", \"vm_test_report_hash\": ");
json_sha256(binding.vm_report_hash);
raw(", \"local_attestation_hash\": ");
json_sha256(binding.local_attestation_hash);
raw(", \"local_approval_hash\": ");
json_sha256(binding.local_approval_hash);
raw(", \"pre_load_service_inventory_hash\": ");
json_sha256(binding.pre_load_service_inventory_hash);
raw(", \"cleanup_actions_hash\": ");
json_sha256(binding.cleanup_actions_hash);
raw("}");
}),
}

fn emit_module_audit_rollback_reference_binding(
    kind: &str,
    binding: &event_log::ModuleAuditRollbackReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_AUDIT_ROLLBACK_REFERENCE_BINDING_FIELDS,
        emit_module_audit_rollback_reference_binding_value,
    );
}

define_direct_binding_fields! { ModuleServiceSlotReservationBindingField,
MODULE_SERVICE_SLOT_RESERVATION_BINDING_FIELDS,
emit_module_service_slot_reservation_binding_value,
event_log::ModuleServiceSlotReservation,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.module_service_slot_reservation.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_load_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.module.load_ephemeral\"");
}),
(LoadMode,
"load_mode",
{ raw("\"ram_only\"");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(GrantsCapability,
"grants_capability",
{ raw("false");
}),
(GrantsLoadNow,
"grants_load_now",
{ raw("false");
}),
(AuthorizesGuestLoad,
"authorizes_guest_load",
{ raw("false");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedComputedGrantReferenceEventId,
"retained_computed_grant_reference_event_id",
{ json_event_id(binding.retained_reference_event_id);
}),
(RetainedAuditRollbackReferenceEventId,
"retained_audit_rollback_reference_event_id",
{ json_event_id(binding.retained_audit_rollback_reference_event_id);
}),
(RamOnlyServiceSlotId,
"ram_only_service_slot_id",
{ json_str(binding.ram_only_service_slot_id.as_str());
}),
(Hashes,
"hashes",
{ raw("{\"reservation_hash\": ");
json_sha256(binding.reservation_hash);
raw(", \"computed_capability_grant_hash\": ");
json_sha256(binding.computed_grant_hash);
raw(", \"audit_record_hash\": ");
json_sha256(binding.audit_record_hash);
raw(", \"rollback_plan_hash\": ");
json_sha256(binding.rollback_plan_hash);
raw(", \"pre_load_service_inventory_hash\": ");
json_sha256(binding.pre_load_service_inventory_hash);
raw("}");
}),
}

fn emit_module_service_slot_reservation_binding(
    kind: &str,
    binding: &event_log::ModuleServiceSlotReservation,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_SERVICE_SLOT_RESERVATION_BINDING_FIELDS,
        emit_module_service_slot_reservation_binding_value,
    );
}

define_direct_binding_fields! { ModuleServiceSlotAllocatorFactSourceEvidenceBindingField,
MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCE_EVIDENCE_BINDING_FIELDS,
emit_module_service_slot_allocator_fact_source_evidence_binding_value,
event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(FactSchema,
"fact_schema",
{ json_str(binding.fact_schema);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.module.load_ephemeral\"");
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(FactId,
"fact_id",
{ json_str(binding.fact_id);
}),
(FactStatus,
"fact_status",
{ json_str(binding.fact_status);
}),
(FactReason,
"fact_reason",
{ json_str(binding.fact_reason);
}),
(FactPresent,
"fact_present",
{ raw_bool(binding.fact_present);
}),
(RetainedServiceSlotReservationPresent,
"retained_service_slot_reservation_present",
{ raw_bool(binding.retained_service_slot_reservation_present);
}),
(AllocatorRuntimeSourceEvidencePresent,
"allocator_runtime_source_evidence_present",
{ raw_bool(binding.allocator_runtime_source_evidence_present);
}),
(RetainedServiceSlotReservationEventId,
"retained_service_slot_reservation_event_id",
{ json_event_id_option(binding.retained_service_slot_reservation_event_id);
}),
(AllocatorRuntimeSourceEvidenceEventId,
"allocator_runtime_source_evidence_event_id",
{ json_event_id_option(binding.allocator_runtime_source_evidence_event_id);
}),
(BindsRetainedServiceSlotReservation,
"binds_retained_service_slot_reservation",
{ raw_bool(binding.binds_retained_service_slot_reservation);
}),
(BindsAllocatorRuntime,
"binds_allocator_runtime",
{ raw_bool(binding.binds_allocator_runtime);
}),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
}

fn emit_module_service_slot_allocator_fact_source_evidence_binding(
    kind: &str,
    binding: &event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCE_EVIDENCE_BINDING_FIELDS,
        emit_module_service_slot_allocator_fact_source_evidence_binding_value,
    );
}

define_direct_binding_fields! { ModuleServiceSlotAllocatorPrerequisiteSourceEvidenceBindingField,
MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCE_EVIDENCE_BINDING_FIELDS,
emit_module_service_slot_allocator_prerequisite_source_evidence_binding_value,
event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(PrerequisiteSchema,
"prerequisite_schema",
{ json_str(binding.prerequisite_schema);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.module.load_ephemeral\"");
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(PrerequisiteId,
"prerequisite_id",
{ json_str(binding.prerequisite_id);
}),
(PrerequisiteStatus,
"prerequisite_status",
{ json_str(binding.prerequisite_status);
}),
(PrerequisiteReason,
"prerequisite_reason",
{ json_str(binding.prerequisite_reason);
}),
(PrerequisiteAvailable,
"prerequisite_available",
{ raw_bool(binding.prerequisite_available);
}),
(RetainedServiceSlotReservationPresent,
"retained_service_slot_reservation_present",
{ raw_bool(binding.retained_service_slot_reservation_present);
}),
(AllocatorRuntimeAvailable,
"allocator_runtime_available",
{ raw_bool(binding.allocator_runtime_available);
}),
(RegistryBindingAvailable,
"registry_binding_available",
{ raw_bool(binding.registry_binding_available);
}),
(HealthStateAvailable,
"health_state_available",
{ raw_bool(binding.health_state_available);
}),
(UnloadCleanupAvailable,
"unload_cleanup_available",
{ raw_bool(binding.unload_cleanup_available);
}),
(AllocatorRuntimeSourceEvidenceEventId,
"allocator_runtime_source_evidence_event_id",
{ json_event_id_option(binding.allocator_runtime_source_evidence_event_id);
}),
(RegistryBindingSourceEvidenceEventId,
"registry_binding_source_evidence_event_id",
{ json_event_id_option(binding.registry_binding_source_evidence_event_id);
}),
(HealthStateSourceEvidenceEventId,
"health_state_source_evidence_event_id",
{ json_event_id_option(binding.health_state_source_evidence_event_id);
}),
(UnloadCleanupSourceEvidenceEventId,
"unload_cleanup_source_evidence_event_id",
{ json_event_id_option(binding.unload_cleanup_source_evidence_event_id);
}),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
}

fn emit_module_service_slot_allocator_prerequisite_source_evidence_binding(
    kind: &str,
    binding: &event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCE_EVIDENCE_BINDING_FIELDS,
        emit_module_service_slot_allocator_prerequisite_source_evidence_binding_value,
    );
}

define_direct_binding_fields! { ModuleServiceSlotAllocatorAuthoritySourceEvidenceBindingField,
MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_BINDING_FIELDS,
emit_module_service_slot_allocator_authority_source_evidence_binding_value,
event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(AuthoritySchema,
"authority_schema",
{ json_str(binding.authority_schema);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.module.load_ephemeral\"");
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(AuthorityId,
"authority_id",
{ json_str(binding.authority_id);
}),
(AuthorityStatus,
"authority_status",
{ json_str(binding.authority_status);
}),
(AuthorityReason,
"authority_reason",
{ json_str(binding.authority_reason);
}),
(AuthorityPresent,
"authority_present",
{ raw_bool(binding.authority_present);
}),
(SourceChainComplete,
"source_chain_complete",
{ raw_bool(binding.source_chain_complete);
}),
(AllocatorRuntimeSourceEvidenceEventId,
"allocator_runtime_source_evidence_event_id",
{ json_event_id_option(binding.allocator_runtime_source_evidence_event_id);
}),
(RegistryBindingSourceEvidenceEventId,
"registry_binding_source_evidence_event_id",
{ json_event_id_option(binding.registry_binding_source_evidence_event_id);
}),
(HealthStateSourceEvidenceEventId,
"health_state_source_evidence_event_id",
{ json_event_id_option(binding.health_state_source_evidence_event_id);
}),
(UnloadCleanupSourceEvidenceEventId,
"unload_cleanup_source_evidence_event_id",
{ json_event_id_option(binding.unload_cleanup_source_evidence_event_id);
}),
(DurableAuditSourceEvidenceEventId,
"durable_audit_source_evidence_event_id",
{ json_event_id_option(binding.durable_audit_source_evidence_event_id);
}),
(RollbackInstallSourceEvidenceEventId,
"rollback_install_source_evidence_event_id",
{ json_event_id_option(binding.rollback_install_source_evidence_event_id);
}),
(ModuleLoaderSourceEvidenceEventId,
"module_loader_source_evidence_event_id",
{ json_event_id_option(binding.module_loader_source_evidence_event_id);
}),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(FutureAuthorityInputsNamed,
"future_authority_inputs_named",
{ raw("true");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
}

fn emit_module_service_slot_allocator_authority_source_evidence_binding(
    kind: &str,
    binding: &event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_BINDING_FIELDS,
        emit_module_service_slot_allocator_authority_source_evidence_binding_value,
    );
}

define_direct_binding_fields! { ModuleServiceSlotAllocationIntentSourceEvidenceBindingField,
MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_BINDING_FIELDS,
emit_module_service_slot_allocation_intent_source_evidence_binding_value,
event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(IntentSchema,
"intent_schema",
{ json_str(binding.intent_schema);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ json_str(binding.requested_capability);
}),
(LoadMode,
"load_mode",
{ json_str(binding.load_mode);
}),
(Target,
"target",
{ json_str(binding.target);
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(IntentId,
"intent_id",
{ json_str(binding.intent_id);
}),
(IntentStatus,
"intent_status",
{ json_str(binding.intent_status);
}),
(IntentReason,
"intent_reason",
{ json_str(binding.intent_reason);
}),
(IntentPresent,
"intent_present",
{ raw_bool(binding.intent_present);
}),
(IntentScope,
"intent_scope",
{ json_str(binding.intent_scope);
}),
(SourceChainComplete,
"source_chain_complete",
{ raw_bool(binding.source_chain_complete);
}),
(ManifestReferenceEventId,
"manifest_reference_event_id",
{ json_event_id_option(binding.manifest_reference_event_id);
}),
(CandidateArtifactReferenceEventId,
"candidate_artifact_reference_event_id",
{ json_event_id_option(binding.artifact_reference_event_id);
}),
(VmTestReportReferenceEventId,
"vm_test_report_reference_event_id",
{ json_event_id_option(binding.vm_report_reference_event_id);
}),
(LocalAttestationReferenceEventId,
"local_attestation_reference_event_id",
{ json_event_id_option(binding.local_attestation_reference_event_id);
}),
(LocalApprovalReferenceEventId,
"local_approval_reference_event_id",
{ json_event_id_option(binding.local_approval_reference_event_id);
}),
(ComputedGrantReferenceEventId,
"computed_grant_reference_event_id",
{ json_event_id_option(binding.computed_grant_reference_event_id);
}),
(AuditRollbackReferenceEventId,
"audit_rollback_reference_event_id",
{ json_event_id_option(binding.audit_rollback_reference_event_id);
}),
(ServiceSlotReservationEventId,
"service_slot_reservation_event_id",
{ json_event_id_option(binding.service_slot_reservation_event_id);
}),
(AllocatorAuthoritySourceEvidenceEventId,
"allocator_authority_source_evidence_event_id",
{ json_event_id_option(binding.allocator_authority_source_evidence_event_id);
}),
(RamOnlyServiceSlotId,
"ram_only_service_slot_id",
{ if let Some(id) = binding.ram_only_service_slot_id { json_str(id.as_str());
} else { raw("null");
} }),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
}

fn emit_module_service_slot_allocation_intent_source_evidence_binding(
    kind: &str,
    binding: &event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_BINDING_FIELDS,
        emit_module_service_slot_allocation_intent_source_evidence_binding_value,
    );
}

define_direct_binding_fields! { ModuleServiceSlotAuthorityInputSourceEvidenceBindingField,
MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCE_EVIDENCE_BINDING_FIELDS,
emit_module_service_slot_authority_input_source_evidence_binding_value,
event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(InputSchema,
"input_schema",
{ json_str(binding.input_schema);
}),
(InputId,
"input_id",
{ json_str(binding.input_id);
}),
(InputName,
"input_name",
{ json_str(binding.input_name);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ json_str(binding.requested_capability);
}),
(LoadMode,
"load_mode",
{ json_str(binding.load_mode);
}),
(Target,
"target",
{ json_str(binding.target);
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(InputStatus,
"input_status",
{ json_str(binding.input_status);
}),
(InputReason,
"input_reason",
{ json_str(binding.input_reason);
}),
(InputPresent,
"input_present",
{ raw_bool(binding.input_present);
}),
(InputScope,
"input_scope",
{ json_str(binding.input_scope);
}),
(DependencySchema,
"dependency_schema",
{ json_str(binding.dependency_schema);
}),
(DependencySourceEvidenceEventId,
"dependency_source_evidence_event_id",
{ json_event_id_option(binding.dependency_source_evidence_event_id);
}),
(DependencyPresent,
"dependency_present",
{ raw_bool(binding.dependency_present);
}),
(RetainedModuleEvidencePresent,
"retained_module_evidence_present",
{ raw_bool(binding.retained_module_evidence_present);
}),
(RetainedServiceSlotReservationPresent,
"retained_service_slot_reservation_present",
{ raw_bool(binding.retained_service_slot_reservation_present);
}),
(AllocatorAuthorityPresent,
"allocator_authority_present",
{ raw_bool(binding.allocator_authority_present);
}),
(AllocationIntentSourceEvidenceEventId,
"allocation_intent_source_evidence_event_id",
{ json_event_id_option(binding.allocation_intent_source_evidence_event_id);
}),
(SourceChainComplete,
"source_chain_complete",
{ raw_bool(binding.source_chain_complete);
}),
(ServiceSlotReservationEventId,
"service_slot_reservation_event_id",
{ json_event_id_option(binding.service_slot_reservation_event_id);
}),
(AllocatorAuthoritySourceEvidenceEventId,
"allocator_authority_source_evidence_event_id",
{ json_event_id_option(binding.allocator_authority_source_evidence_event_id);
}),
(RamOnlyServiceSlotId,
"ram_only_service_slot_id",
{ if let Some(id) = binding.ram_only_service_slot_id { json_str(id.as_str());
} else { raw("null");
} }),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
}

fn emit_module_service_slot_authority_input_source_evidence_binding(
    kind: &str,
    binding: &event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCE_EVIDENCE_BINDING_FIELDS,
        emit_module_service_slot_authority_input_source_evidence_binding_value,
    );
}

define_direct_binding_fields! { ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidenceBindingField,
MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_BINDING_FIELDS,
emit_module_service_slot_allocator_authority_decision_source_evidence_binding_value,
event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(DecisionSchema,
"decision_schema",
{ json_str(binding.decision_schema);
}),
(DecisionId,
"decision_id",
{ json_str(binding.decision_id);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ json_str(binding.requested_capability);
}),
(LoadMode,
"load_mode",
{ json_str(binding.load_mode);
}),
(Target,
"target",
{ json_str(binding.target);
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(DecisionStatus,
"decision_status",
{ json_str(binding.decision_status);
}),
(DecisionReason,
"decision_reason",
{ json_str(binding.decision_reason);
}),
(DecisionPresent,
"decision_present",
{ raw_bool(binding.decision_present);
}),
(DecisionScope,
"decision_scope",
{ json_str(binding.decision_scope);
}),
(AllocatorAuthorityPresent,
"allocator_authority_present",
{ raw_bool(binding.allocator_authority_present);
}),
(AllocationIntentPresent,
"allocation_intent_present",
{ raw_bool(binding.allocation_intent_present);
}),
(AuthorityInputsComplete,
"authority_inputs_complete",
{ raw_bool(binding.authority_inputs_complete);
}),
(SourceChainComplete,
"source_chain_complete",
{ raw_bool(binding.source_chain_complete);
}),
(AllocatorAuthoritySourceEvidenceEventId,
"allocator_authority_source_evidence_event_id",
{ json_event_id_option(binding.allocator_authority_source_evidence_event_id);
}),
(AllocationIntentSourceEvidenceEventId,
"allocation_intent_source_evidence_event_id",
{ json_event_id_option(binding.allocation_intent_source_evidence_event_id);
}),
(AuthorityInputSourceEvidenceEventIds,
"authority_input_source_evidence_event_ids",
{ raw("[");
let mut idx = 0usize;
while idx < binding.authority_input_source_evidence_event_ids.len() { json_event_id_option(binding.authority_input_source_evidence_event_ids[idx]);
if idx + 1 != binding.authority_input_source_evidence_event_ids.len() { raw(", ");
} idx += 1;
} raw("]");
}),
(AuthorityInputPresent,
"authority_input_present",
{ raw("[");
let mut idx = 0usize;
while idx < binding.authority_input_present.len() { raw_bool(binding.authority_input_present[idx]);
if idx + 1 != binding.authority_input_present.len() { raw(", ");
} idx += 1;
} raw("]");
}),
(RetainedModuleEvidencePresent,
"retained_module_evidence_present",
{ raw_bool(binding.retained_module_evidence_present);
}),
(RetainedServiceSlotReservationPresent,
"retained_service_slot_reservation_present",
{ raw_bool(binding.retained_service_slot_reservation_present);
}),
(ServiceSlotReservationEventId,
"service_slot_reservation_event_id",
{ json_event_id_option(binding.service_slot_reservation_event_id);
}),
(RamOnlyServiceSlotId,
"ram_only_service_slot_id",
{ if let Some(id) = binding.ram_only_service_slot_id { json_str(id.as_str());
} else { raw("null");
} }),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
}

fn emit_module_service_slot_allocator_authority_decision_source_evidence_binding(
    kind: &str,
    binding: &event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_BINDING_FIELDS,
        emit_module_service_slot_allocator_authority_decision_source_evidence_binding_value,
    );
}

define_direct_binding_fields! { ModuleServiceSlotRegistryWriteCommitGateSourceEvidenceBindingField,
MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_BINDING_FIELDS,
emit_module_service_slot_registry_write_commit_gate_source_evidence_binding_value,
event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(GateSchema,
"gate_schema",
{ json_str(binding.gate_schema);
}),
(GateId,
"gate_id",
{ json_str(binding.gate_id);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ json_str(binding.requested_capability);
}),
(LoadMode,
"load_mode",
{ json_str(binding.load_mode);
}),
(Target,
"target",
{ json_str(binding.target);
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(GateStatus,
"gate_status",
{ json_str(binding.gate_status);
}),
(GateReason,
"gate_reason",
{ json_str(binding.gate_reason);
}),
(GatePresent,
"gate_present",
{ raw_bool(binding.gate_present);
}),
(GateScope,
"gate_scope",
{ json_str(binding.gate_scope);
}),
(GateSchemaOk,
"gate_schema_ok",
{ raw_bool(binding.gate_schema_ok);
}),
(GateProvenanceOk,
"gate_provenance_ok",
{ raw_bool(binding.gate_provenance_ok);
}),
(GateClassification,
"gate_classification",
{ json_str(binding.gate_classification);
}),
(AuthorityDecisionPresent,
"authority_decision_present",
{ raw_bool(binding.authority_decision_present);
}),
(RegistryWriteAuthorityPresent,
"registry_write_authority_present",
{ raw_bool(binding.registry_write_authority_present);
}),
(RegistryBindingAvailable,
"registry_binding_available",
{ raw_bool(binding.registry_binding_available);
}),
(DurableAuditWriteAvailable,
"durable_audit_write_available",
{ raw_bool(binding.durable_audit_write_available);
}),
(RollbackPlanInstallAvailable,
"rollback_plan_install_available",
{ raw_bool(binding.rollback_plan_install_available);
}),
(RetainedServiceSlotReservationPresent,
"retained_service_slot_reservation_present",
{ raw_bool(binding.retained_service_slot_reservation_present);
}),
(SourceChainComplete,
"source_chain_complete",
{ raw_bool(binding.source_chain_complete);
}),
(AuthorityDecisionSourceEvidenceEventId,
"authority_decision_source_evidence_event_id",
{ json_event_id_option(binding.authority_decision_source_evidence_event_id);
}),
(RegistryWriteAuthoritySourceEvidenceEventId,
"registry_write_authority_source_evidence_event_id",
{ json_event_id_option(binding.registry_write_authority_source_evidence_event_id);
}),
(RegistryBindingSourceEvidenceEventId,
"registry_binding_source_evidence_event_id",
{ json_event_id_option(binding.registry_binding_source_evidence_event_id);
}),
(DurableAuditSourceEvidenceEventId,
"durable_audit_source_evidence_event_id",
{ json_event_id_option(binding.durable_audit_source_evidence_event_id);
}),
(RollbackInstallSourceEvidenceEventId,
"rollback_install_source_evidence_event_id",
{ json_event_id_option(binding.rollback_install_source_evidence_event_id);
}),
(ServiceSlotReservationEventId,
"service_slot_reservation_event_id",
{ json_event_id_option(binding.service_slot_reservation_event_id);
}),
(RamOnlyServiceSlotId,
"ram_only_service_slot_id",
{ if let Some(id) = binding.ram_only_service_slot_id { json_str(id.as_str());
} else { raw("null");
} }),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(AuthorizesRegistryWrite,
"authorizes_registry_write",
{ raw_bool(binding.authorizes_registry_write);
}),
(MutatesServiceRegistry,
"mutates_service_registry",
{ raw_bool(binding.mutates_service_registry);
}),
(WritesDurableAuditState,
"writes_durable_audit_state",
{ raw_bool(binding.writes_durable_audit_state);
}),
(InstallsRollbackState,
"installs_rollback_state",
{ raw_bool(binding.installs_rollback_state);
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw_bool(binding.allocates_service_slot);
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadsArtifact,
"loads_artifact",
{ raw_bool(binding.loads_artifact);
}),
(LoadAttempted,
"load_attempted",
{ raw_bool(binding.loads_artifact);
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
}

fn emit_module_service_slot_registry_write_commit_gate_source_evidence_binding(
    kind: &str,
    binding: &event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_BINDING_FIELDS,
        emit_module_service_slot_registry_write_commit_gate_source_evidence_binding_value,
    );
}

define_direct_binding_fields! { ModuleLoaderIdentitySourceEvidenceBindingField,
MODULE_LOADER_IDENTITY_SOURCE_EVIDENCE_BINDING_FIELDS,
emit_module_loader_identity_source_evidence_binding_value,
event_log::ModuleLoaderIdentitySourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(FactSchema,
"fact_schema",
{ json_str(binding.fact_schema);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.module.load_ephemeral\"");
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(FactId,
"fact_id",
{ json_str(binding.fact_id);
}),
(IdentityStatus,
"identity_status",
{ json_str(binding.identity_status);
}),
(IdentityReason,
"identity_reason",
{ json_str(binding.identity_reason);
}),
(IdentityPresent,
"identity_present",
{ raw_bool(binding.identity_present);
}),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(AcceptsLoaderDescriptor,
"accepts_loader_descriptor",
{ raw("false");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw("false");
}),
(LoadsArtifact,
"loads_artifact",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
(RetainedModuleEvidencePresent,
"retained_module_evidence_present",
{ raw_bool(binding.retained_module_evidence_present);
}),
(ServiceSlotAllocatorReadinessPresent,
"service_slot_allocator_readiness_present",
{ raw_bool(binding.service_slot_allocator_readiness_present);
}),
(ServiceSlotAllocatorReady,
"service_slot_allocator_ready",
{ raw_bool(binding.service_slot_allocator_ready);
}),
(AuditRollbackWriteBoundaryPresent,
"audit_rollback_write_boundary_present",
{ raw_bool(binding.audit_rollback_write_boundary_present);
}),
(RetainedModuleEvidenceEventIds,
"retained_module_evidence_event_ids",
{ raw("{\"manifest_reference_event_id\": ");
json_event_id_option(binding.manifest_reference_event_id);
raw(", \"candidate_artifact_reference_event_id\": ");
json_event_id_option(binding.artifact_reference_event_id);
raw(", \"vm_test_report_reference_event_id\": ");
json_event_id_option(binding.vm_test_report_reference_event_id);
raw(", \"local_attestation_reference_event_id\": ");
json_event_id_option(binding.local_attestation_reference_event_id);
raw(", \"local_approval_reference_event_id\": ");
json_event_id_option(binding.local_approval_reference_event_id);
raw(", \"computed_grant_reference_event_id\": ");
json_event_id_option(binding.computed_grant_reference_event_id);
raw(", \"audit_rollback_reference_event_id\": ");
json_event_id_option(binding.audit_rollback_reference_event_id);
raw(", \"service_slot_reservation_event_id\": ");
json_event_id_option(binding.service_slot_reservation_event_id);
raw("}");
}),
}

fn emit_module_loader_identity_source_evidence_binding(
    kind: &str,
    binding: &event_log::ModuleLoaderIdentitySourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_LOADER_IDENTITY_SOURCE_EVIDENCE_BINDING_FIELDS,
        emit_module_loader_identity_source_evidence_binding_value,
    );
}

define_direct_binding_fields! { ModuleLoaderArtifactHashBindingSourceEvidenceBindingField,
MODULE_LOADER_ARTIFACT_HASH_BINDING_SOURCE_EVIDENCE_BINDING_FIELDS,
emit_module_loader_artifact_hash_binding_source_evidence_binding_value,
event_log::ModuleLoaderArtifactHashBindingSourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(FactSchema,
"fact_schema",
{ json_str(binding.fact_schema);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.module.load_ephemeral\"");
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(FactId,
"fact_id",
{ json_str(binding.fact_id);
}),
(ArtifactHashBindingStatus,
"artifact_hash_binding_status",
{ json_str(binding.artifact_hash_binding_status);
}),
(ArtifactHashBindingReason,
"artifact_hash_binding_reason",
{ json_str(binding.artifact_hash_binding_reason);
}),
(ArtifactHashBindingPresent,
"artifact_hash_binding_present",
{ raw_bool(binding.artifact_hash_binding_present);
}),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(AcceptsLoaderDescriptor,
"accepts_loader_descriptor",
{ raw("false");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw("false");
}),
(LoadsArtifact,
"loads_artifact",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
(RetainedModuleEvidencePresent,
"retained_module_evidence_present",
{ raw_bool(binding.retained_module_evidence_present);
}),
(ServiceSlotAllocatorReadinessPresent,
"service_slot_allocator_readiness_present",
{ raw_bool(binding.service_slot_allocator_readiness_present);
}),
(ServiceSlotAllocatorReady,
"service_slot_allocator_ready",
{ raw_bool(binding.service_slot_allocator_ready);
}),
(AuditRollbackWriteBoundaryPresent,
"audit_rollback_write_boundary_present",
{ raw_bool(binding.audit_rollback_write_boundary_present);
}),
(LoaderIdentityPresent,
"loader_identity_present",
{ raw_bool(binding.loader_identity_present);
}),
(BindsLoaderIdentity,
"binds_loader_identity",
{ raw_bool(binding.binds_loader_identity);
}),
(LoaderIdentitySourceEvidenceEventId,
"loader_identity_source_evidence_event_id",
{ json_event_id_option(binding.loader_identity_source_evidence_event_id);
}),
}

fn emit_module_loader_artifact_hash_binding_source_evidence_binding(
    kind: &str,
    binding: &event_log::ModuleLoaderArtifactHashBindingSourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_LOADER_ARTIFACT_HASH_BINDING_SOURCE_EVIDENCE_BINDING_FIELDS,
        emit_module_loader_artifact_hash_binding_source_evidence_binding_value,
    );
}

define_direct_binding_fields! { ModuleLoaderFactSourceEvidenceBindingField,
MODULE_LOADER_FACT_SOURCE_EVIDENCE_BINDING_FIELDS,
emit_module_loader_fact_source_evidence_binding_value,
event_log::ModuleLoaderFactSourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(FactSchema,
"fact_schema",
{ json_str(binding.fact_schema);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.module.load_ephemeral\"");
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(FactId,
"fact_id",
{ json_str(binding.fact_id);
}),
(FactStatus,
"fact_status",
{ json_str(binding.fact_status);
}),
(FactReason,
"fact_reason",
{ json_str(binding.fact_reason);
}),
(FactPresent,
"fact_present",
{ raw_bool(binding.fact_present);
}),
(DependencyGate,
"dependency_gate",
{ json_str(binding.dependency_gate);
}),
(DependencySchema,
"dependency_schema",
{ json_str(binding.dependency_schema);
}),
(DependencyMethod,
"dependency_method",
{ json_str(binding.dependency_method);
}),
(DependencyPresent,
"dependency_present",
{ raw_bool(binding.dependency_present);
}),
(DependencySourceEvidenceEventId,
"dependency_source_evidence_event_id",
{ json_event_id_option(binding.dependency_source_evidence_event_id);
}),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(AcceptsLoaderDescriptor,
"accepts_loader_descriptor",
{ raw("false");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw("false");
}),
(LoadsArtifact,
"loads_artifact",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
(RetainedModuleEvidencePresent,
"retained_module_evidence_present",
{ raw_bool(binding.retained_module_evidence_present);
}),
(ServiceSlotAllocatorReadinessPresent,
"service_slot_allocator_readiness_present",
{ raw_bool(binding.service_slot_allocator_readiness_present);
}),
(ServiceSlotAllocatorReady,
"service_slot_allocator_ready",
{ raw_bool(binding.service_slot_allocator_ready);
}),
(AuditRollbackWriteBoundaryPresent,
"audit_rollback_write_boundary_present",
{ raw_bool(binding.audit_rollback_write_boundary_present);
}),
(BindsRetainedModuleEvidence,
"binds_retained_module_evidence",
{ raw_bool(binding.binds_retained_module_evidence);
}),
(BindsServiceSlotAllocator,
"binds_service_slot_allocator",
{ raw_bool(binding.binds_service_slot_allocator);
}),
(BindsAuditRollbackWriteBoundary,
"binds_audit_rollback_write_boundary",
{ raw_bool(binding.binds_audit_rollback_write_boundary);
}),
(BindsDependency,
"binds_dependency",
{ raw_bool(binding.binds_dependency);
}),
}

fn emit_module_loader_fact_source_evidence_binding(
    kind: &str,
    binding: &event_log::ModuleLoaderFactSourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_LOADER_FACT_SOURCE_EVIDENCE_BINDING_FIELDS,
        emit_module_loader_fact_source_evidence_binding_value,
    );
}

define_direct_binding_fields! { ModuleLoaderRuntimeExecutionCommitGateSourceEvidenceBindingField,
MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_EVIDENCE_BINDING_FIELDS,
emit_module_loader_runtime_execution_commit_gate_source_evidence_binding_value,
event_log::ModuleLoaderRuntimeExecutionCommitGateSourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(GateSchema,
"gate_schema",
{ json_str(binding.gate_schema);
}),
(GateId,
"gate_id",
{ json_str(binding.gate_id);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ json_str(binding.requested_capability);
}),
(LoadMode,
"load_mode",
{ json_str(binding.load_mode);
}),
(Target,
"target",
{ json_str(binding.target);
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(GateStatus,
"gate_status",
{ json_str(binding.gate_status);
}),
(GateReason,
"gate_reason",
{ json_str(binding.gate_reason);
}),
(GatePresent,
"gate_present",
{ raw_bool(binding.gate_present);
}),
(GateScope,
"gate_scope",
{ json_str(binding.gate_scope);
}),
(GateSchemaOk,
"gate_schema_ok",
{ raw_bool(binding.gate_schema_ok);
}),
(GateProvenanceOk,
"gate_provenance_ok",
{ raw_bool(binding.gate_provenance_ok);
}),
(GateClassification,
"gate_classification",
{ json_str(binding.gate_classification);
}),
(AuthorityDecisionPresent,
"authority_decision_present",
{ raw_bool(binding.authority_decision_present);
}),
(LoaderRuntimeContractPresent,
"loader_runtime_contract_present",
{ raw_bool(binding.loader_runtime_contract_present);
}),
(LoaderRuntimeSourceEvidenceComplete,
"loader_runtime_source_evidence_complete",
{ raw_bool(binding.loader_runtime_source_evidence_complete);
}),
(ServiceSlotBindingSourceEvidencePresent,
"service_slot_binding_source_evidence_present",
{ raw_bool(binding.service_slot_binding_source_evidence_present);
}),
(ServiceSlotBindingFactPresent,
"service_slot_binding_fact_present",
{ raw_bool(binding.service_slot_binding_fact_present);
}),
(AuditRollbackWriteBoundarySourceEvidencePresent,
"audit_rollback_write_boundary_source_evidence_present",
{ raw_bool(binding.audit_rollback_write_boundary_source_evidence_present);
}),
(AuditRollbackWriteBoundaryFactPresent,
"audit_rollback_write_boundary_fact_present",
{ raw_bool(binding.audit_rollback_write_boundary_fact_present);
}),
(RetainedServiceSlotReservationPresent,
"retained_service_slot_reservation_present",
{ raw_bool(binding.retained_service_slot_reservation_present);
}),
(SourceChainComplete,
"source_chain_complete",
{ raw_bool(binding.source_chain_complete);
}),
(AuthorityDecisionSourceEvidenceEventId,
"authority_decision_source_evidence_event_id",
{ json_event_id_option(binding.authority_decision_source_evidence_event_id);
}),
(LoaderRuntimeContractSourceEvidenceEventId,
"loader_runtime_contract_source_evidence_event_id",
{ json_event_id_option(binding.loader_runtime_contract_source_evidence_event_id);
}),
(LoaderRuntimeSourceEvidenceEventIds,
"loader_runtime_source_evidence_event_ids",
{ raw("[");
let mut idx = 0usize;
while idx < binding.loader_runtime_source_evidence_event_ids.len() { json_event_id_option(binding.loader_runtime_source_evidence_event_ids[idx]);
if idx + 1 != binding.loader_runtime_source_evidence_event_ids.len() { raw(", ");
} idx += 1;
} raw("]");
}),
(LoaderRuntimeSourceEvidencePresent,
"loader_runtime_source_evidence_present",
{ raw("[");
let mut idx = 0usize;
while idx < binding.loader_runtime_source_evidence_present.len() { raw_bool(binding.loader_runtime_source_evidence_present[idx]);
if idx + 1 != binding.loader_runtime_source_evidence_present.len() { raw(", ");
} idx += 1;
} raw("]");
}),
(LoaderRuntimeFactPresent,
"loader_runtime_fact_present",
{ raw("[");
let mut idx = 0usize;
while idx < binding.loader_runtime_fact_present.len() { raw_bool(binding.loader_runtime_fact_present[idx]);
if idx + 1 != binding.loader_runtime_fact_present.len() { raw(", ");
} idx += 1;
} raw("]");
}),
(ServiceSlotReservationEventId,
"service_slot_reservation_event_id",
{ json_event_id_option(binding.service_slot_reservation_event_id);
}),
(RamOnlyServiceSlotId,
"ram_only_service_slot_id",
{ if let Some(id) = binding.ram_only_service_slot_id { json_str(id.as_str());
} else { raw("null");
} }),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(AcceptsLoaderDescriptor,
"accepts_loader_descriptor",
{ raw_bool(binding.accepts_loader_descriptor);
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw_bool(binding.accepts_artifact_bytes);
}),
(AuthorizesExecution,
"authorizes_execution",
{ raw_bool(binding.authorizes_execution);
}),
(MutatesServiceRegistry,
"mutates_service_registry",
{ raw_bool(binding.mutates_service_registry);
}),
(WritesDurableAuditState,
"writes_durable_audit_state",
{ raw_bool(binding.writes_durable_audit_state);
}),
(InstallsRollbackState,
"installs_rollback_state",
{ raw_bool(binding.installs_rollback_state);
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw_bool(binding.allocates_service_slot);
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadsArtifact,
"loads_artifact",
{ raw_bool(binding.loads_artifact);
}),
(LoadAttempted,
"load_attempted",
{ raw_bool(binding.loads_artifact);
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
}

fn emit_module_loader_runtime_execution_commit_gate_source_evidence_binding(
    kind: &str,
    binding: &event_log::ModuleLoaderRuntimeExecutionCommitGateSourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_EVIDENCE_BINDING_FIELDS,
        emit_module_loader_runtime_execution_commit_gate_source_evidence_binding_value,
    );
}

define_direct_binding_fields! { ModuleLoaderDescriptorIntakeBoundarySourceEvidenceBindingField,
MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_EVIDENCE_BINDING_FIELDS,
emit_module_loader_descriptor_intake_boundary_source_evidence_binding_value,
event_log::ModuleLoaderDescriptorIntakeBoundarySourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(BoundarySchema,
"boundary_schema",
{ json_str(binding.boundary_schema);
}),
(BoundaryId,
"boundary_id",
{ json_str(binding.boundary_id);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ json_str(binding.requested_capability);
}),
(LoadMode,
"load_mode",
{ json_str(binding.load_mode);
}),
(Target,
"target",
{ json_str(binding.target);
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(BoundaryStatus,
"boundary_status",
{ json_str(binding.boundary_status);
}),
(BoundaryReason,
"boundary_reason",
{ json_str(binding.boundary_reason);
}),
(BoundaryPresent,
"boundary_present",
{ raw_bool(binding.boundary_present);
}),
(BoundaryScope,
"boundary_scope",
{ json_str(binding.boundary_scope);
}),
(BoundarySchemaOk,
"boundary_schema_ok",
{ raw_bool(binding.boundary_schema_ok);
}),
(BoundaryProvenanceOk,
"boundary_provenance_ok",
{ raw_bool(binding.boundary_provenance_ok);
}),
(BoundaryClassification,
"boundary_classification",
{ json_str(binding.boundary_classification);
}),
(RegistryWriteCommitGatePresent,
"registry_write_commit_gate_present",
{ raw_bool(binding.registry_write_commit_gate_present);
}),
(ExecutionCommitGatePresent,
"execution_commit_gate_present",
{ raw_bool(binding.execution_commit_gate_present);
}),
(LoaderRuntimeSourceEvidenceComplete,
"loader_runtime_source_evidence_complete",
{ raw_bool(binding.loader_runtime_source_evidence_complete);
}),
(RetainedModuleEvidencePresent,
"retained_module_evidence_present",
{ raw_bool(binding.retained_module_evidence_present);
}),
(RetainedServiceSlotReservationPresent,
"retained_service_slot_reservation_present",
{ raw_bool(binding.retained_service_slot_reservation_present);
}),
(SourceChainComplete,
"source_chain_complete",
{ raw_bool(binding.source_chain_complete);
}),
(RegistryWriteCommitGateSourceEvidenceEventId,
"registry_write_commit_gate_source_evidence_event_id",
{ json_event_id_option(binding.registry_write_commit_gate_source_evidence_event_id);
}),
(ExecutionCommitGateSourceEvidenceEventId,
"execution_commit_gate_source_evidence_event_id",
{ json_event_id_option(binding.execution_commit_gate_source_evidence_event_id);
}),
(LoaderRuntimeSourceEvidenceEventIds,
"loader_runtime_source_evidence_event_ids",
{ raw("[");
let mut idx = 0usize;
while idx < binding.loader_runtime_source_evidence_event_ids.len() { json_event_id_option(binding.loader_runtime_source_evidence_event_ids[idx]);
if idx + 1 != binding.loader_runtime_source_evidence_event_ids.len() { raw(", ");
} idx += 1;
} raw("]");
}),
(LoaderRuntimeSourceEvidencePresent,
"loader_runtime_source_evidence_present",
{ raw("[");
let mut idx = 0usize;
while idx < binding.loader_runtime_source_evidence_present.len() { raw_bool(binding.loader_runtime_source_evidence_present[idx]);
if idx + 1 != binding.loader_runtime_source_evidence_present.len() { raw(", ");
} idx += 1;
} raw("]");
}),
(LoaderRuntimeFactPresent,
"loader_runtime_fact_present",
{ raw("[");
let mut idx = 0usize;
while idx < binding.loader_runtime_fact_present.len() { raw_bool(binding.loader_runtime_fact_present[idx]);
if idx + 1 != binding.loader_runtime_fact_present.len() { raw(", ");
} idx += 1;
} raw("]");
}),
(RetainedModuleEvidenceEventIds,
"retained_module_evidence_event_ids",
{ raw("{\"manifest_reference_event_id\": ");
json_event_id_option(binding.manifest_reference_event_id);
raw(", \"candidate_artifact_reference_event_id\": ");
json_event_id_option(binding.artifact_reference_event_id);
raw(", \"vm_test_report_reference_event_id\": ");
json_event_id_option(binding.vm_test_report_reference_event_id);
raw(", \"local_attestation_reference_event_id\": ");
json_event_id_option(binding.local_attestation_reference_event_id);
raw(", \"local_approval_reference_event_id\": ");
json_event_id_option(binding.local_approval_reference_event_id);
raw(", \"computed_grant_reference_event_id\": ");
json_event_id_option(binding.computed_grant_reference_event_id);
raw(", \"audit_rollback_reference_event_id\": ");
json_event_id_option(binding.audit_rollback_reference_event_id);
raw(", \"service_slot_reservation_event_id\": ");
json_event_id_option(binding.service_slot_reservation_event_id);
raw("}");
}),
(RamOnlyServiceSlotId,
"ram_only_service_slot_id",
{ if let Some(id) = binding.ram_only_service_slot_id { json_str(id.as_str());
} else { raw("null");
} }),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(AcceptsLoaderDescriptor,
"accepts_loader_descriptor",
{ raw_bool(binding.accepts_loader_descriptor);
}),
(AcceptsDescriptorBytes,
"accepts_descriptor_bytes",
{ raw_bool(binding.accepts_descriptor_bytes);
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw_bool(binding.accepts_artifact_bytes);
}),
(AuthorizesDescriptorIntake,
"authorizes_descriptor_intake",
{ raw_bool(binding.authorizes_descriptor_intake);
}),
(AuthorizesExecution,
"authorizes_execution",
{ raw_bool(binding.authorizes_execution);
}),
(MutatesServiceRegistry,
"mutates_service_registry",
{ raw_bool(binding.mutates_service_registry);
}),
(WritesDurableAuditState,
"writes_durable_audit_state",
{ raw_bool(binding.writes_durable_audit_state);
}),
(InstallsRollbackState,
"installs_rollback_state",
{ raw_bool(binding.installs_rollback_state);
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw_bool(binding.allocates_service_slot);
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadsArtifact,
"loads_artifact",
{ raw_bool(binding.loads_artifact);
}),
(LoadAttempted,
"load_attempted",
{ raw_bool(binding.loads_artifact);
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
}

fn emit_module_loader_descriptor_intake_boundary_source_evidence_binding(
    kind: &str,
    binding: &event_log::ModuleLoaderDescriptorIntakeBoundarySourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_EVIDENCE_BINDING_FIELDS,
        emit_module_loader_descriptor_intake_boundary_source_evidence_binding_value,
    );
}

define_direct_binding_fields! { ModuleLoaderArtifactByteIntakeBoundarySourceEvidenceBindingField,
MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_EVIDENCE_BINDING_FIELDS,
emit_module_loader_artifact_byte_intake_boundary_source_evidence_binding_value,
event_log::ModuleLoaderArtifactByteIntakeBoundarySourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(BoundarySchema,
"boundary_schema",
{ json_str(binding.boundary_schema);
}),
(BoundaryId,
"boundary_id",
{ json_str(binding.boundary_id);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ json_str(binding.requested_capability);
}),
(LoadMode,
"load_mode",
{ json_str(binding.load_mode);
}),
(Target,
"target",
{ json_str(binding.target);
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(BoundaryStatus,
"boundary_status",
{ json_str(binding.boundary_status);
}),
(BoundaryReason,
"boundary_reason",
{ json_str(binding.boundary_reason);
}),
(BoundaryPresent,
"boundary_present",
{ raw_bool(binding.boundary_present);
}),
(BoundaryScope,
"boundary_scope",
{ json_str(binding.boundary_scope);
}),
(BoundarySchemaOk,
"boundary_schema_ok",
{ raw_bool(binding.boundary_schema_ok);
}),
(BoundaryProvenanceOk,
"boundary_provenance_ok",
{ raw_bool(binding.boundary_provenance_ok);
}),
(BoundaryClassification,
"boundary_classification",
{ json_str(binding.boundary_classification);
}),
(DescriptorIntakeBoundaryPresent,
"descriptor_intake_boundary_present",
{ raw_bool(binding.descriptor_intake_boundary_present);
}),
(DescriptorIntakeBoundarySourceChainComplete,
"descriptor_intake_boundary_source_chain_complete",
{ raw_bool(binding.descriptor_intake_boundary_source_chain_complete);
}),
(ExecutionCommitGatePresent,
"execution_commit_gate_present",
{ raw_bool(binding.execution_commit_gate_present);
}),
(ArtifactHashBindingPresent,
"artifact_hash_binding_present",
{ raw_bool(binding.artifact_hash_binding_present);
}),
(RetainedArtifactReferencePresent,
"retained_artifact_reference_present",
{ raw_bool(binding.retained_artifact_reference_present);
}),
(RetainedModuleEvidencePresent,
"retained_module_evidence_present",
{ raw_bool(binding.retained_module_evidence_present);
}),
(RetainedServiceSlotReservationPresent,
"retained_service_slot_reservation_present",
{ raw_bool(binding.retained_service_slot_reservation_present);
}),
(SourceChainComplete,
"source_chain_complete",
{ raw_bool(binding.source_chain_complete);
}),
(DescriptorIntakeBoundarySourceEvidenceEventId,
"descriptor_intake_boundary_source_evidence_event_id",
{ json_event_id_option(binding.descriptor_intake_boundary_source_evidence_event_id);
}),
(ExecutionCommitGateSourceEvidenceEventId,
"execution_commit_gate_source_evidence_event_id",
{ json_event_id_option(binding.execution_commit_gate_source_evidence_event_id);
}),
(ArtifactHashBindingSourceEvidenceEventId,
"artifact_hash_binding_source_evidence_event_id",
{ json_event_id_option(binding.artifact_hash_binding_source_evidence_event_id);
}),
(LoaderRuntimeSourceEvidenceEventIds,
"loader_runtime_source_evidence_event_ids",
{ raw("[");
let mut idx = 0usize;
while idx < binding.loader_runtime_source_evidence_event_ids.len() { json_event_id_option(binding.loader_runtime_source_evidence_event_ids[idx]);
if idx + 1 != binding.loader_runtime_source_evidence_event_ids.len() { raw(", ");
} idx += 1;
} raw("]");
}),
(LoaderRuntimeSourceEvidencePresent,
"loader_runtime_source_evidence_present",
{ raw("[");
let mut idx = 0usize;
while idx < binding.loader_runtime_source_evidence_present.len() { raw_bool(binding.loader_runtime_source_evidence_present[idx]);
if idx + 1 != binding.loader_runtime_source_evidence_present.len() { raw(", ");
} idx += 1;
} raw("]");
}),
(LoaderRuntimeFactPresent,
"loader_runtime_fact_present",
{ raw("[");
let mut idx = 0usize;
while idx < binding.loader_runtime_fact_present.len() { raw_bool(binding.loader_runtime_fact_present[idx]);
if idx + 1 != binding.loader_runtime_fact_present.len() { raw(", ");
} idx += 1;
} raw("]");
}),
(RetainedModuleEvidenceEventIds,
"retained_module_evidence_event_ids",
{ raw("{\"manifest_reference_event_id\": ");
json_event_id_option(binding.manifest_reference_event_id);
raw(", \"candidate_artifact_reference_event_id\": ");
json_event_id_option(binding.artifact_reference_event_id);
raw(", \"vm_test_report_reference_event_id\": ");
json_event_id_option(binding.vm_test_report_reference_event_id);
raw(", \"local_attestation_reference_event_id\": ");
json_event_id_option(binding.local_attestation_reference_event_id);
raw(", \"local_approval_reference_event_id\": ");
json_event_id_option(binding.local_approval_reference_event_id);
raw(", \"computed_grant_reference_event_id\": ");
json_event_id_option(binding.computed_grant_reference_event_id);
raw(", \"audit_rollback_reference_event_id\": ");
json_event_id_option(binding.audit_rollback_reference_event_id);
raw(", \"service_slot_reservation_event_id\": ");
json_event_id_option(binding.service_slot_reservation_event_id);
raw("}");
}),
(RamOnlyServiceSlotId,
"ram_only_service_slot_id",
{ if let Some(id) = binding.ram_only_service_slot_id { json_str(id.as_str());
} else { raw("null");
} }),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(AcceptsLoaderDescriptor,
"accepts_loader_descriptor",
{ raw_bool(binding.accepts_loader_descriptor);
}),
(AcceptsDescriptorBytes,
"accepts_descriptor_bytes",
{ raw_bool(binding.accepts_descriptor_bytes);
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw_bool(binding.accepts_artifact_bytes);
}),
(AuthorizesDescriptorIntake,
"authorizes_descriptor_intake",
{ raw_bool(binding.authorizes_descriptor_intake);
}),
(AuthorizesArtifactByteIntake,
"authorizes_artifact_byte_intake",
{ raw_bool(binding.authorizes_artifact_byte_intake);
}),
(AuthorizesExecution,
"authorizes_execution",
{ raw_bool(binding.authorizes_execution);
}),
(MutatesServiceRegistry,
"mutates_service_registry",
{ raw_bool(binding.mutates_service_registry);
}),
(WritesDurableAuditState,
"writes_durable_audit_state",
{ raw_bool(binding.writes_durable_audit_state);
}),
(InstallsRollbackState,
"installs_rollback_state",
{ raw_bool(binding.installs_rollback_state);
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw_bool(binding.allocates_service_slot);
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadsArtifact,
"loads_artifact",
{ raw_bool(binding.loads_artifact);
}),
(LoadAttempted,
"load_attempted",
{ raw_bool(binding.loads_artifact);
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
}

fn emit_module_loader_artifact_byte_intake_boundary_source_evidence_binding(
    kind: &str,
    binding: &event_log::ModuleLoaderArtifactByteIntakeBoundarySourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_EVIDENCE_BINDING_FIELDS,
        emit_module_loader_artifact_byte_intake_boundary_source_evidence_binding_value,
    );
}

define_direct_binding_fields! { ModuleLoaderExecutionAuthorizationBoundarySourceEvidenceBindingField,
MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_BINDING_FIELDS,
emit_module_loader_execution_authorization_boundary_source_evidence_binding_value,
event_log::ModuleLoaderExecutionAuthorizationBoundarySourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(BoundarySchema,
"boundary_schema",
{ json_str(binding.boundary_schema);
}),
(BoundaryId,
"boundary_id",
{ json_str(binding.boundary_id);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ json_str(binding.requested_capability);
}),
(LoadMode,
"load_mode",
{ json_str(binding.load_mode);
}),
(Target,
"target",
{ json_str(binding.target);
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(BoundaryStatus,
"boundary_status",
{ json_str(binding.boundary_status);
}),
(BoundaryReason,
"boundary_reason",
{ json_str(binding.boundary_reason);
}),
(BoundaryPresent,
"boundary_present",
{ raw_bool(binding.boundary_present);
}),
(BoundaryScope,
"boundary_scope",
{ json_str(binding.boundary_scope);
}),
(BoundarySchemaOk,
"boundary_schema_ok",
{ raw_bool(binding.boundary_schema_ok);
}),
(BoundaryProvenanceOk,
"boundary_provenance_ok",
{ raw_bool(binding.boundary_provenance_ok);
}),
(BoundaryClassification,
"boundary_classification",
{ json_str(binding.boundary_classification);
}),
(SourceChainComplete,
"source_chain_complete",
{ raw_bool(binding.source_chain_complete);
}),
(ArtifactByteIntakeBoundaryPresent,
"artifact_byte_intake_boundary_present",
{ raw_bool(binding.artifact_byte_intake_boundary_present);
}),
(DescriptorIntakeBoundaryPresent,
"descriptor_intake_boundary_present",
{ raw_bool(binding.descriptor_intake_boundary_present);
}),
(ExecutionCommitGatePresent,
"execution_commit_gate_present",
{ raw_bool(binding.execution_commit_gate_present);
}),
(EntrypointAbiSourceEvidencePresent,
"entrypoint_abi_source_evidence_present",
{ raw_bool(binding.entrypoint_abi_source_evidence_present);
}),
(AddressSpaceSourceEvidencePresent,
"address_space_source_evidence_present",
{ raw_bool(binding.address_space_source_evidence_present);
}),
(MemoryMapSourceEvidencePresent,
"memory_map_source_evidence_present",
{ raw_bool(binding.memory_map_source_evidence_present);
}),
(AuditRollbackWriteBoundarySourceEvidencePresent,
"audit_rollback_write_boundary_source_evidence_present",
{ raw_bool(binding.audit_rollback_write_boundary_source_evidence_present);
}),
(RetainedModuleEvidencePresent,
"retained_module_evidence_present",
{ raw_bool(binding.retained_module_evidence_present);
}),
(RetainedServiceSlotReservationPresent,
"retained_service_slot_reservation_present",
{ raw_bool(binding.retained_service_slot_reservation_present);
}),
(ArtifactByteIntakeBoundarySourceEvidenceEventId,
"artifact_byte_intake_boundary_source_evidence_event_id",
{ json_event_id_option(binding.artifact_byte_intake_boundary_source_evidence_event_id);
}),
(DescriptorIntakeBoundarySourceEvidenceEventId,
"descriptor_intake_boundary_source_evidence_event_id",
{ json_event_id_option(binding.descriptor_intake_boundary_source_evidence_event_id);
}),
(ExecutionCommitGateSourceEvidenceEventId,
"execution_commit_gate_source_evidence_event_id",
{ json_event_id_option(binding.execution_commit_gate_source_evidence_event_id);
}),
(EntrypointAbiSourceEvidenceEventId,
"entrypoint_abi_source_evidence_event_id",
{ json_event_id_option(binding.entrypoint_abi_source_evidence_event_id);
}),
(AddressSpaceSourceEvidenceEventId,
"address_space_source_evidence_event_id",
{ json_event_id_option(binding.address_space_source_evidence_event_id);
}),
(MemoryMapSourceEvidenceEventId,
"memory_map_source_evidence_event_id",
{ json_event_id_option(binding.memory_map_source_evidence_event_id);
}),
(AuditRollbackWriteBoundarySourceEvidenceEventId,
"audit_rollback_write_boundary_source_evidence_event_id",
{ json_event_id_option(binding.audit_rollback_write_boundary_source_evidence_event_id);
}),
(RetainedModuleEvidenceEventIds,
"retained_module_evidence_event_ids",
{ raw("{\"manifest_reference_event_id\": ");
json_event_id_option(binding.manifest_reference_event_id);
raw(", \"candidate_artifact_reference_event_id\": ");
json_event_id_option(binding.artifact_reference_event_id);
raw(", \"vm_test_report_reference_event_id\": ");
json_event_id_option(binding.vm_test_report_reference_event_id);
raw(", \"local_attestation_reference_event_id\": ");
json_event_id_option(binding.local_attestation_reference_event_id);
raw(", \"local_approval_reference_event_id\": ");
json_event_id_option(binding.local_approval_reference_event_id);
raw(", \"computed_grant_reference_event_id\": ");
json_event_id_option(binding.computed_grant_reference_event_id);
raw(", \"audit_rollback_reference_event_id\": ");
json_event_id_option(binding.audit_rollback_reference_event_id);
raw(", \"service_slot_reservation_event_id\": ");
json_event_id_option(binding.service_slot_reservation_event_id);
raw("}");
}),
(RamOnlyServiceSlotId,
"ram_only_service_slot_id",
{ if let Some(id) = binding.ram_only_service_slot_id { json_str(id.as_str());
} else { raw("null");
} }),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(AcceptsLoaderDescriptor,
"accepts_loader_descriptor",
{ raw_bool(binding.accepts_loader_descriptor);
}),
(AcceptsDescriptorBytes,
"accepts_descriptor_bytes",
{ raw_bool(binding.accepts_descriptor_bytes);
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw_bool(binding.accepts_artifact_bytes);
}),
(AuthorizesDescriptorIntake,
"authorizes_descriptor_intake",
{ raw_bool(binding.authorizes_descriptor_intake);
}),
(AuthorizesArtifactByteIntake,
"authorizes_artifact_byte_intake",
{ raw_bool(binding.authorizes_artifact_byte_intake);
}),
(MapsExecutablePages,
"maps_executable_pages",
{ raw_bool(binding.maps_executable_pages);
}),
(JumpsToEntrypoint,
"jumps_to_entrypoint",
{ raw_bool(binding.jumps_to_entrypoint);
}),
(AuthorizesExecution,
"authorizes_execution",
{ raw_bool(binding.authorizes_execution);
}),
(MutatesServiceRegistry,
"mutates_service_registry",
{ raw_bool(binding.mutates_service_registry);
}),
(WritesDurableAuditState,
"writes_durable_audit_state",
{ raw_bool(binding.writes_durable_audit_state);
}),
(InstallsRollbackState,
"installs_rollback_state",
{ raw_bool(binding.installs_rollback_state);
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw_bool(binding.allocates_service_slot);
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadsArtifact,
"loads_artifact",
{ raw_bool(binding.loads_artifact);
}),
(LoadAttempted,
"load_attempted",
{ raw_bool(binding.loads_artifact);
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
}

fn emit_module_loader_execution_authorization_boundary_source_evidence_binding(
    kind: &str,
    binding: &event_log::ModuleLoaderExecutionAuthorizationBoundarySourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_BINDING_FIELDS,
        emit_module_loader_execution_authorization_boundary_source_evidence_binding_value,
    );
}

define_direct_binding_fields! { ModuleLoaderServiceRegistryMutationBoundarySourceEvidenceBindingField,
MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_EVIDENCE_BINDING_FIELDS,
emit_module_loader_service_registry_mutation_boundary_source_evidence_binding_value,
event_log::ModuleLoaderServiceRegistryMutationBoundarySourceEvidence,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(BoundarySchema,
"boundary_schema",
{ json_str(binding.boundary_schema);
}),
(BoundaryId,
"boundary_id",
{ json_str(binding.boundary_id);
}),
(Status,
"status",
{ json_str(binding.readiness_status);
}),
(Reason,
"reason",
{ json_str(binding.readiness_reason);
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ json_str(binding.requested_capability);
}),
(LoadMode,
"load_mode",
{ json_str(binding.load_mode);
}),
(Target,
"target",
{ json_str(binding.target);
}),
(SourceMethod,
"source_method",
{ json_str(binding.source_method);
}),
(SourceFactLocator,
"source_fact_locator",
{ json_str(binding.source_fact_locator);
}),
(BoundaryStatus,
"boundary_status",
{ json_str(binding.boundary_status);
}),
(BoundaryReason,
"boundary_reason",
{ json_str(binding.boundary_reason);
}),
(BoundaryPresent,
"boundary_present",
{ raw_bool(binding.boundary_present);
}),
(BoundaryScope,
"boundary_scope",
{ json_str(binding.boundary_scope);
}),
(BoundarySchemaOk,
"boundary_schema_ok",
{ raw_bool(binding.boundary_schema_ok);
}),
(BoundaryProvenanceOk,
"boundary_provenance_ok",
{ raw_bool(binding.boundary_provenance_ok);
}),
(BoundaryClassification,
"boundary_classification",
{ json_str(binding.boundary_classification);
}),
(SourceChainComplete,
"source_chain_complete",
{ raw_bool(binding.source_chain_complete);
}),
(ExecutionAuthorizationBoundaryPresent,
"execution_authorization_boundary_present",
{ raw_bool(binding.execution_authorization_boundary_present);
}),
(ExecutionAuthorizationBoundarySourceChainComplete,
"execution_authorization_boundary_source_chain_complete",
{ raw_bool(binding.execution_authorization_boundary_source_chain_complete);
}),
(RegistryWriteCommitGatePresent,
"registry_write_commit_gate_present",
{ raw_bool(binding.registry_write_commit_gate_present);
}),
(ServiceSlotBindingSourceEvidencePresent,
"service_slot_binding_source_evidence_present",
{ raw_bool(binding.service_slot_binding_source_evidence_present);
}),
(RetainedModuleEvidencePresent,
"retained_module_evidence_present",
{ raw_bool(binding.retained_module_evidence_present);
}),
(RetainedServiceSlotReservationPresent,
"retained_service_slot_reservation_present",
{ raw_bool(binding.retained_service_slot_reservation_present);
}),
(ExecutionAuthorizationBoundarySourceEvidenceEventId,
"execution_authorization_boundary_source_evidence_event_id",
{ json_event_id_option(binding.execution_authorization_boundary_source_evidence_event_id);
}),
(RegistryWriteCommitGateSourceEvidenceEventId,
"registry_write_commit_gate_source_evidence_event_id",
{ json_event_id_option(binding.registry_write_commit_gate_source_evidence_event_id);
}),
(ServiceSlotBindingSourceEvidenceEventId,
"service_slot_binding_source_evidence_event_id",
{ json_event_id_option(binding.service_slot_binding_source_evidence_event_id);
}),
(RetainedModuleEvidenceEventIds,
"retained_module_evidence_event_ids",
{ raw("{\"manifest_reference_event_id\": ");
json_event_id_option(binding.manifest_reference_event_id);
raw(", \"candidate_artifact_reference_event_id\": ");
json_event_id_option(binding.artifact_reference_event_id);
raw(", \"vm_test_report_reference_event_id\": ");
json_event_id_option(binding.vm_test_report_reference_event_id);
raw(", \"local_attestation_reference_event_id\": ");
json_event_id_option(binding.local_attestation_reference_event_id);
raw(", \"local_approval_reference_event_id\": ");
json_event_id_option(binding.local_approval_reference_event_id);
raw(", \"computed_grant_reference_event_id\": ");
json_event_id_option(binding.computed_grant_reference_event_id);
raw(", \"audit_rollback_reference_event_id\": ");
json_event_id_option(binding.audit_rollback_reference_event_id);
raw(", \"service_slot_reservation_event_id\": ");
json_event_id_option(binding.service_slot_reservation_event_id);
raw("}");
}),
(RamOnlyServiceSlotId,
"ram_only_service_slot_id",
{ if let Some(id) = binding.ram_only_service_slot_id { json_str(id.as_str());
} else { raw("null");
} }),
(SourceEvidenceRetained,
"source_evidence_retained",
{ raw("true");
}),
(Retention,
"retention",
{ raw("\"current_boot_ram_event_log\"");
}),
(AcceptsLoaderDescriptor,
"accepts_loader_descriptor",
{ raw_bool(binding.accepts_loader_descriptor);
}),
(AcceptsDescriptorBytes,
"accepts_descriptor_bytes",
{ raw_bool(binding.accepts_descriptor_bytes);
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw_bool(binding.accepts_artifact_bytes);
}),
(AuthorizesDescriptorIntake,
"authorizes_descriptor_intake",
{ raw_bool(binding.authorizes_descriptor_intake);
}),
(AuthorizesArtifactByteIntake,
"authorizes_artifact_byte_intake",
{ raw_bool(binding.authorizes_artifact_byte_intake);
}),
(MapsExecutablePages,
"maps_executable_pages",
{ raw_bool(binding.maps_executable_pages);
}),
(JumpsToEntrypoint,
"jumps_to_entrypoint",
{ raw_bool(binding.jumps_to_entrypoint);
}),
(AuthorizesExecution,
"authorizes_execution",
{ raw_bool(binding.authorizes_execution);
}),
(MutatesServiceRegistry,
"mutates_service_registry",
{ raw_bool(binding.mutates_service_registry);
}),
(WritesDurableAuditState,
"writes_durable_audit_state",
{ raw_bool(binding.writes_durable_audit_state);
}),
(InstallsRollbackState,
"installs_rollback_state",
{ raw_bool(binding.installs_rollback_state);
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw_bool(binding.allocates_service_slot);
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw_bool(binding.creates_service_inventory_records);
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(CanLoadNow,
"can_load_now",
{ raw("false");
}),
(LoadsArtifact,
"loads_artifact",
{ raw_bool(binding.loads_artifact);
}),
(LoadAttempted,
"load_attempted",
{ raw_bool(binding.loads_artifact);
}),
(AuthorizesLoad,
"authorizes_load",
{ raw("false");
}),
}

fn emit_module_loader_service_registry_mutation_boundary_source_evidence_binding(
    kind: &str,
    binding: &event_log::ModuleLoaderServiceRegistryMutationBoundarySourceEvidence,
) {
    emit_binding_object_direct(
        binding,
        kind,
        MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_EVIDENCE_BINDING_FIELDS,
        emit_module_loader_service_registry_mutation_boundary_source_evidence_binding_value,
    );
}

define_direct_binding_fields! { RecoveryArtifactIdentityReferenceBindingField,
RECOVERY_ARTIFACT_IDENTITY_REFERENCE_BINDING_FIELDS,
emit_recovery_artifact_identity_reference_binding_value,
event_log::RecoveryArtifactIdentityReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_artifact_identity.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_load_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.load_artifact\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(LoadsNormalModule,
"loads_normal_module",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(Hashes,
"hashes",
{ raw("{\"identity_reference_hash\": ");
json_sha256(binding.identity_reference_hash);
raw(", \"artifact_hash\": ");
json_sha256(binding.artifact_hash);
raw("}");
}),
}

fn emit_recovery_artifact_identity_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryArtifactIdentityReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_ARTIFACT_IDENTITY_REFERENCE_BINDING_FIELDS,
        emit_recovery_artifact_identity_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryArtifactTrustReferenceBindingField,
RECOVERY_ARTIFACT_TRUST_REFERENCE_BINDING_FIELDS,
emit_recovery_artifact_trust_reference_binding_value,
event_log::RecoveryArtifactTrustReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_artifact_trust.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_load_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.load_artifact\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(LoadsNormalModule,
"loads_normal_module",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryArtifactIdentityEventId,
"retained_recovery_artifact_identity_event_id",
{ json_event_id(binding.retained_identity_reference_event_id);
}),
(Hashes,
"hashes",
{ raw("{\"trust_reference_hash\": ");
json_sha256(binding.trust_reference_hash);
raw(", \"identity_reference_hash\": ");
json_sha256(binding.identity_reference_hash);
raw(", \"artifact_hash\": ");
json_sha256(binding.artifact_hash);
raw(", \"trust_hash\": ");
json_sha256(binding.trust_hash);
raw("}");
}),
}

fn emit_recovery_artifact_trust_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryArtifactTrustReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_ARTIFACT_TRUST_REFERENCE_BINDING_FIELDS,
        emit_recovery_artifact_trust_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryArtifactVmTestReferenceBindingField,
RECOVERY_ARTIFACT_VM_TEST_REFERENCE_BINDING_FIELDS,
emit_recovery_artifact_vm_test_reference_binding_value,
event_log::RecoveryArtifactVmTestReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_artifact_vm_test.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_load_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.load_artifact\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsVmTestJson,
"accepts_vm_test_json",
{ raw("false");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(LoadsNormalModule,
"loads_normal_module",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryArtifactIdentityEventId,
"retained_recovery_artifact_identity_event_id",
{ json_event_id(binding.retained_identity_reference_event_id);
}),
(RetainedRecoveryArtifactTrustEventId,
"retained_recovery_artifact_trust_event_id",
{ json_event_id(binding.retained_trust_reference_event_id);
}),
(Hashes,
"hashes",
{ raw("{\"vm_test_reference_hash\": ");
json_sha256(binding.vm_test_reference_hash);
raw(", \"identity_reference_hash\": ");
json_sha256(binding.identity_reference_hash);
raw(", \"trust_reference_hash\": ");
json_sha256(binding.trust_reference_hash);
raw(", \"artifact_hash\": ");
json_sha256(binding.artifact_hash);
raw(", \"trust_hash\": ");
json_sha256(binding.trust_hash);
raw(", \"vm_test_hash\": ");
json_sha256(binding.vm_test_hash);
raw("}");
}),
}

fn emit_recovery_artifact_vm_test_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryArtifactVmTestReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_ARTIFACT_VM_TEST_REFERENCE_BINDING_FIELDS,
        emit_recovery_artifact_vm_test_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryArtifactLocalApprovalReferenceBindingField,
RECOVERY_ARTIFACT_LOCAL_APPROVAL_REFERENCE_BINDING_FIELDS,
emit_recovery_artifact_local_approval_reference_binding_value,
event_log::RecoveryArtifactLocalApprovalReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_artifact_local_approval.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_load_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.load_artifact\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsLocalApprovalText,
"accepts_local_approval_text",
{ raw("false");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(LoadsNormalModule,
"loads_normal_module",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryArtifactIdentityEventId,
"retained_recovery_artifact_identity_event_id",
{ json_event_id(binding.retained_identity_reference_event_id);
}),
(RetainedRecoveryArtifactTrustEventId,
"retained_recovery_artifact_trust_event_id",
{ json_event_id(binding.retained_trust_reference_event_id);
}),
(RetainedRecoveryArtifactVmTestEventId,
"retained_recovery_artifact_vm_test_event_id",
{ json_event_id(binding.retained_vm_test_reference_event_id);
}),
(Hashes,
"hashes",
{ raw("{\"local_approval_reference_hash\": ");
json_sha256(binding.local_approval_reference_hash);
raw(", \"identity_reference_hash\": ");
json_sha256(binding.identity_reference_hash);
raw(", \"trust_reference_hash\": ");
json_sha256(binding.trust_reference_hash);
raw(", \"vm_test_reference_hash\": ");
json_sha256(binding.vm_test_reference_hash);
raw(", \"artifact_hash\": ");
json_sha256(binding.artifact_hash);
raw(", \"trust_hash\": ");
json_sha256(binding.trust_hash);
raw(", \"vm_test_hash\": ");
json_sha256(binding.vm_test_hash);
raw(", \"local_approval_hash\": ");
json_sha256(binding.local_approval_hash);
raw("}");
}),
}

fn emit_recovery_artifact_local_approval_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryArtifactLocalApprovalReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_ARTIFACT_LOCAL_APPROVAL_REFERENCE_BINDING_FIELDS,
        emit_recovery_artifact_local_approval_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryArtifactLoaderReferenceBindingField,
RECOVERY_ARTIFACT_LOADER_REFERENCE_BINDING_FIELDS,
emit_recovery_artifact_loader_reference_binding_value,
event_log::RecoveryArtifactLoaderReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_artifact_loader.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_load_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.load_artifact\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsLoaderDescriptor,
"accepts_loader_descriptor",
{ raw("false");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw("false");
}),
(LoadsRecoveryLoader,
"loads_recovery_loader",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(LoadsNormalModule,
"loads_normal_module",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryArtifactIdentityEventId,
"retained_recovery_artifact_identity_event_id",
{ json_event_id(binding.retained_identity_reference_event_id);
}),
(RetainedRecoveryArtifactTrustEventId,
"retained_recovery_artifact_trust_event_id",
{ json_event_id(binding.retained_trust_reference_event_id);
}),
(RetainedRecoveryArtifactVmTestEventId,
"retained_recovery_artifact_vm_test_event_id",
{ json_event_id(binding.retained_vm_test_reference_event_id);
}),
(RetainedRecoveryArtifactLocalApprovalEventId,
"retained_recovery_artifact_local_approval_event_id",
{ json_event_id(binding.retained_local_approval_reference_event_id);
}),
(Hashes,
"hashes",
{ raw("{\"loader_reference_hash\": ");
json_sha256(binding.loader_reference_hash);
raw(", \"identity_reference_hash\": ");
json_sha256(binding.identity_reference_hash);
raw(", \"trust_reference_hash\": ");
json_sha256(binding.trust_reference_hash);
raw(", \"vm_test_reference_hash\": ");
json_sha256(binding.vm_test_reference_hash);
raw(", \"local_approval_reference_hash\": ");
json_sha256(binding.local_approval_reference_hash);
raw(", \"artifact_hash\": ");
json_sha256(binding.artifact_hash);
raw(", \"trust_hash\": ");
json_sha256(binding.trust_hash);
raw(", \"vm_test_hash\": ");
json_sha256(binding.vm_test_hash);
raw(", \"local_approval_hash\": ");
json_sha256(binding.local_approval_hash);
raw(", \"loader_hash\": ");
json_sha256(binding.loader_hash);
raw("}");
}),
}

fn emit_recovery_artifact_loader_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryArtifactLoaderReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_ARTIFACT_LOADER_REFERENCE_BINDING_FIELDS,
        emit_recovery_artifact_loader_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryArtifactRollbackEvidenceReferenceBindingField,
RECOVERY_ARTIFACT_ROLLBACK_EVIDENCE_REFERENCE_BINDING_FIELDS,
emit_recovery_artifact_rollback_evidence_reference_binding_value,
event_log::RecoveryArtifactRollbackEvidenceReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_artifact_rollback_evidence.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_load_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.load_artifact\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsRollbackEvidenceJson,
"accepts_rollback_evidence_json",
{ raw("false");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(LoadsNormalModule,
"loads_normal_module",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryArtifactIdentityEventId,
"retained_recovery_artifact_identity_event_id",
{ json_event_id(binding.retained_identity_reference_event_id);
}),
(RetainedRecoveryArtifactTrustEventId,
"retained_recovery_artifact_trust_event_id",
{ json_event_id(binding.retained_trust_reference_event_id);
}),
(RetainedRecoveryArtifactVmTestEventId,
"retained_recovery_artifact_vm_test_event_id",
{ json_event_id(binding.retained_vm_test_reference_event_id);
}),
(RetainedRecoveryArtifactLocalApprovalEventId,
"retained_recovery_artifact_local_approval_event_id",
{ json_event_id(binding.retained_local_approval_reference_event_id);
}),
(RetainedRecoveryArtifactLoaderEventId,
"retained_recovery_artifact_loader_event_id",
{ json_event_id(binding.retained_loader_reference_event_id);
}),
(Hashes,
"hashes",
{ raw("{\"rollback_evidence_reference_hash\": ");
json_sha256(binding.rollback_evidence_reference_hash);
raw(", \"identity_reference_hash\": ");
json_sha256(binding.identity_reference_hash);
raw(", \"trust_reference_hash\": ");
json_sha256(binding.trust_reference_hash);
raw(", \"vm_test_reference_hash\": ");
json_sha256(binding.vm_test_reference_hash);
raw(", \"local_approval_reference_hash\": ");
json_sha256(binding.local_approval_reference_hash);
raw(", \"loader_reference_hash\": ");
json_sha256(binding.loader_reference_hash);
raw(", \"artifact_hash\": ");
json_sha256(binding.artifact_hash);
raw(", \"trust_hash\": ");
json_sha256(binding.trust_hash);
raw(", \"vm_test_hash\": ");
json_sha256(binding.vm_test_hash);
raw(", \"local_approval_hash\": ");
json_sha256(binding.local_approval_hash);
raw(", \"loader_hash\": ");
json_sha256(binding.loader_hash);
raw(", \"rollback_evidence_hash\": ");
json_sha256(binding.rollback_evidence_hash);
raw("}");
}),
}

fn emit_recovery_artifact_rollback_evidence_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryArtifactRollbackEvidenceReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_ARTIFACT_ROLLBACK_EVIDENCE_REFERENCE_BINDING_FIELDS,
        emit_recovery_artifact_rollback_evidence_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryLifelineRequestReferenceBindingField,
RECOVERY_LIFELINE_REQUEST_REFERENCE_BINDING_FIELDS,
emit_recovery_lifeline_request_reference_binding_value,
event_log::RecoveryLifelineRequestReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_lifeline_request.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_load_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.load_artifact\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsLifelineRequestJson,
"accepts_lifeline_request_json",
{ raw("false");
}),
(AcceptsLoaderDescriptor,
"accepts_loader_descriptor",
{ raw("false");
}),
(AcceptsArtifactBytes,
"accepts_artifact_bytes",
{ raw("false");
}),
(LoadsRecoveryLoader,
"loads_recovery_loader",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(LoadsNormalModule,
"loads_normal_module",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryArtifactIdentityEventId,
"retained_recovery_artifact_identity_event_id",
{ json_event_id(binding.retained_identity_reference_event_id);
}),
(RetainedRecoveryArtifactTrustEventId,
"retained_recovery_artifact_trust_event_id",
{ json_event_id(binding.retained_trust_reference_event_id);
}),
(RetainedRecoveryArtifactVmTestEventId,
"retained_recovery_artifact_vm_test_event_id",
{ json_event_id(binding.retained_vm_test_reference_event_id);
}),
(RetainedRecoveryArtifactLocalApprovalEventId,
"retained_recovery_artifact_local_approval_event_id",
{ json_event_id(binding.retained_local_approval_reference_event_id);
}),
(RetainedRecoveryArtifactLoaderEventId,
"retained_recovery_artifact_loader_event_id",
{ json_event_id(binding.retained_loader_reference_event_id);
}),
(RetainedRecoveryArtifactRollbackEvidenceEventId,
"retained_recovery_artifact_rollback_evidence_event_id",
{ json_event_id(binding.retained_rollback_evidence_reference_event_id);
}),
(Hashes,
"hashes",
{ raw("{\"lifeline_request_reference_hash\": ");
json_sha256(binding.lifeline_request_reference_hash);
raw(", \"identity_reference_hash\": ");
json_sha256(binding.identity_reference_hash);
raw(", \"trust_reference_hash\": ");
json_sha256(binding.trust_reference_hash);
raw(", \"vm_test_reference_hash\": ");
json_sha256(binding.vm_test_reference_hash);
raw(", \"local_approval_reference_hash\": ");
json_sha256(binding.local_approval_reference_hash);
raw(", \"loader_reference_hash\": ");
json_sha256(binding.loader_reference_hash);
raw(", \"rollback_evidence_reference_hash\": ");
json_sha256(binding.rollback_evidence_reference_hash);
raw(", \"artifact_hash\": ");
json_sha256(binding.artifact_hash);
raw(", \"trust_hash\": ");
json_sha256(binding.trust_hash);
raw(", \"vm_test_hash\": ");
json_sha256(binding.vm_test_hash);
raw(", \"local_approval_hash\": ");
json_sha256(binding.local_approval_hash);
raw(", \"loader_hash\": ");
json_sha256(binding.loader_hash);
raw(", \"rollback_evidence_hash\": ");
json_sha256(binding.rollback_evidence_hash);
raw("}");
}),
}

fn emit_recovery_lifeline_request_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryLifelineRequestReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_LIFELINE_REQUEST_REFERENCE_BINDING_FIELDS,
        emit_recovery_lifeline_request_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryLifelineCommandEnvelopeReferenceBindingField,
RECOVERY_LIFELINE_COMMAND_ENVELOPE_REFERENCE_BINDING_FIELDS,
emit_recovery_lifeline_command_envelope_reference_binding_value,
event_log::RecoveryLifelineCommandEnvelopeReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_lifeline_command_envelope_reference.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryLifelineRequestEventId,
"retained_recovery_lifeline_request_event_id",
{ json_event_id(binding.retained_lifeline_request_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(RequiredCapability,
"required_capability",
{ json_str(binding.required_capability);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandAdmissionBoundaryId,
"command_admission_boundary_id",
{ json_str(binding.command_admission_boundary_id);
}),
(Hashes,
"hashes",
{ raw("{\"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"lifeline_request_reference_hash\": ");
json_sha256(binding.lifeline_request_reference_hash);
raw("}");
}),
}

fn emit_recovery_lifeline_command_envelope_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryLifelineCommandEnvelopeReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_LIFELINE_COMMAND_ENVELOPE_REFERENCE_BINDING_FIELDS,
        emit_recovery_lifeline_command_envelope_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryLifelineCommandBodyCanonicalizationReferenceBindingField,
RECOVERY_LIFELINE_COMMAND_BODY_CANONICALIZATION_REFERENCE_BINDING_FIELDS,
emit_recovery_lifeline_command_body_canonicalization_reference_binding_value,
event_log::RecoveryLifelineCommandBodyCanonicalizationReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_lifeline_command_body_canonicalization.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryLifelineCommandEnvelopeEventId,
"retained_recovery_lifeline_command_envelope_event_id",
{ json_event_id(binding.retained_command_envelope_reference_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(Hashes,
"hashes",
{ raw("{\"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw("}");
}),
}

fn emit_recovery_lifeline_command_body_canonicalization_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryLifelineCommandBodyCanonicalizationReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_LIFELINE_COMMAND_BODY_CANONICALIZATION_REFERENCE_BINDING_FIELDS,
        emit_recovery_lifeline_command_body_canonicalization_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryLifelineCommandHandlerBindingReferenceBindingField,
RECOVERY_LIFELINE_COMMAND_HANDLER_BINDING_REFERENCE_BINDING_FIELDS,
emit_recovery_lifeline_command_handler_binding_reference_binding_value,
event_log::RecoveryLifelineCommandHandlerBindingReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_lifeline_command_handler_binding.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryLifelineCommandBodyCanonicalizationEventId,
"retained_recovery_lifeline_command_body_canonicalization_event_id",
{ json_event_id(binding.retained_command_body_canonicalization_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(HandlerId,
"handler_id",
{ json_str(binding.handler_id);
}),
(Hashes,
"hashes",
{ raw("{\"handler_binding_hash\": ");
json_sha256(binding.handler_binding_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"handler_input_binding_hash\": ");
json_sha256(binding.handler_input_binding_hash);
raw("}");
}),
}

fn emit_recovery_lifeline_command_handler_binding_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryLifelineCommandHandlerBindingReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_LIFELINE_COMMAND_HANDLER_BINDING_REFERENCE_BINDING_FIELDS,
        emit_recovery_lifeline_command_handler_binding_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryLifelineStatusReadHandlerReferenceBindingField,
RECOVERY_LIFELINE_STATUS_READ_HANDLER_REFERENCE_BINDING_FIELDS,
emit_recovery_lifeline_status_read_handler_reference_binding_value,
event_log::RecoveryLifelineStatusReadHandlerReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_lifeline_status_read_handler.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(ExecutesLifelineStatus,
"executes_lifeline_status",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryLifelineCommandHandlerBindingEventId,
"retained_recovery_lifeline_command_handler_binding_event_id",
{ json_event_id(binding.retained_command_handler_binding_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(StatusHandlerId,
"status_handler_id",
{ json_str(binding.status_handler_id);
}),
(Hashes,
"hashes",
{ raw("{\"status_read_handler_hash\": ");
json_sha256(binding.status_read_handler_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"handler_binding_hash\": ");
json_sha256(binding.handler_binding_hash);
raw(", \"status_read_projection_hash\": ");
json_sha256(binding.status_read_projection_hash);
raw("}");
}),
}

fn emit_recovery_lifeline_status_read_handler_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryLifelineStatusReadHandlerReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_LIFELINE_STATUS_READ_HANDLER_REFERENCE_BINDING_FIELDS,
        emit_recovery_lifeline_status_read_handler_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryRollbackPreviewAuthorizationReferenceBindingField,
RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_REFERENCE_BINDING_FIELDS,
emit_recovery_rollback_preview_authorization_reference_binding_value,
event_log::RecoveryRollbackPreviewAuthorizationReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_rollback_preview_authorization.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(ExecutesLifelineStatus,
"executes_lifeline_status",
{ raw("false");
}),
(ExecutesRollbackPreview,
"executes_rollback_preview",
{ raw("false");
}),
(ExecutesRollbackApply,
"executes_rollback_apply",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryLifelineStatusReadHandlerEventId,
"retained_recovery_lifeline_status_read_handler_event_id",
{ json_event_id(binding.retained_status_read_handler_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(RollbackPreviewAuthorizationId,
"rollback_preview_authorization_id",
{ json_str(binding.rollback_preview_authorization_id);
}),
(Hashes,
"hashes",
{ raw("{\"rollback_preview_authorization_hash\": ");
json_sha256(binding.rollback_preview_authorization_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"handler_binding_hash\": ");
json_sha256(binding.handler_binding_hash);
raw(", \"status_read_handler_hash\": ");
json_sha256(binding.status_read_handler_hash);
raw(", \"rollback_preview_projection_hash\": ");
json_sha256(binding.rollback_preview_projection_hash);
raw("}");
}),
}

fn emit_recovery_rollback_preview_authorization_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryRollbackPreviewAuthorizationReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_REFERENCE_BINDING_FIELDS,
        emit_recovery_rollback_preview_authorization_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryRollbackApplyAuthorizationReferenceBindingField,
RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_REFERENCE_BINDING_FIELDS,
emit_recovery_rollback_apply_authorization_reference_binding_value,
event_log::RecoveryRollbackApplyAuthorizationReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_rollback_apply_authorization.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(ExecutesLifelineStatus,
"executes_lifeline_status",
{ raw("false");
}),
(ExecutesRollbackPreview,
"executes_rollback_preview",
{ raw("false");
}),
(ExecutesRollbackApply,
"executes_rollback_apply",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryRollbackPreviewAuthorizationEventId,
"retained_recovery_rollback_preview_authorization_event_id",
{ json_event_id(binding.retained_rollback_preview_authorization_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(RollbackApplyAuthorizationId,
"rollback_apply_authorization_id",
{ json_str(binding.rollback_apply_authorization_id);
}),
(Hashes,
"hashes",
{ raw("{\"rollback_apply_authorization_hash\": ");
json_sha256(binding.rollback_apply_authorization_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"handler_binding_hash\": ");
json_sha256(binding.handler_binding_hash);
raw(", \"status_read_handler_hash\": ");
json_sha256(binding.status_read_handler_hash);
raw(", \"rollback_preview_authorization_hash\": ");
json_sha256(binding.rollback_preview_authorization_hash);
raw(", \"rollback_apply_projection_hash\": ");
json_sha256(binding.rollback_apply_projection_hash);
raw(", \"source_rollback_apply_denial_hash\": ");
json_sha256(binding.source_rollback_apply_denial_hash);
raw(", \"source_durable_policy_write_authority_decision_hash\": ");
json_sha256(binding.source_durable_policy_write_authority_decision_hash);
raw(", \"source_recovery_rollback_inspect_source_reference_hash\": ");
json_sha256(binding.source_recovery_rollback_inspect_source_reference_hash);
raw("}");
}),
}

fn emit_recovery_rollback_apply_authorization_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryRollbackApplyAuthorizationReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_REFERENCE_BINDING_FIELDS,
        emit_recovery_rollback_apply_authorization_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryDisableModuleTargetBindingReferenceBindingField,
RECOVERY_DISABLE_MODULE_TARGET_BINDING_REFERENCE_BINDING_FIELDS,
emit_recovery_disable_module_target_binding_reference_binding_value,
event_log::RecoveryDisableModuleTargetBindingReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_disable_module_target_binding.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(ExecutesLifelineStatus,
"executes_lifeline_status",
{ raw("false");
}),
(ExecutesRollbackPreview,
"executes_rollback_preview",
{ raw("false");
}),
(ExecutesRollbackApply,
"executes_rollback_apply",
{ raw("false");
}),
(ExecutesDisableModule,
"executes_disable_module",
{ raw("false");
}),
(DisablesModule,
"disables_module",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryRollbackApplyAuthorizationEventId,
"retained_recovery_rollback_apply_authorization_event_id",
{ json_event_id(binding.retained_rollback_apply_authorization_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(DisableModuleTargetId,
"disable_module_target_id",
{ json_str(binding.disable_module_target_id);
}),
(Hashes,
"hashes",
{ raw("{\"disable_module_target_binding_hash\": ");
json_sha256(binding.disable_module_target_binding_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"handler_binding_hash\": ");
json_sha256(binding.handler_binding_hash);
raw(", \"status_read_handler_hash\": ");
json_sha256(binding.status_read_handler_hash);
raw(", \"rollback_preview_authorization_hash\": ");
json_sha256(binding.rollback_preview_authorization_hash);
raw(", \"rollback_apply_authorization_hash\": ");
json_sha256(binding.rollback_apply_authorization_hash);
raw(", \"source_rollback_apply_denial_hash\": ");
json_sha256(binding.source_rollback_apply_denial_hash);
raw(", \"source_durable_policy_write_authority_decision_hash\": ");
json_sha256(binding.source_durable_policy_write_authority_decision_hash);
raw(", \"source_recovery_rollback_inspect_source_reference_hash\": ");
json_sha256(binding.source_recovery_rollback_inspect_source_reference_hash);
raw(", \"disable_module_target_projection_hash\": ");
json_sha256(binding.disable_module_target_projection_hash);
raw("}");
}),
}

fn emit_recovery_disable_module_target_binding_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryDisableModuleTargetBindingReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_DISABLE_MODULE_TARGET_BINDING_REFERENCE_BINDING_FIELDS,
        emit_recovery_disable_module_target_binding_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryRestartLastGoodTargetBindingReferenceBindingField,
RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_REFERENCE_BINDING_FIELDS,
emit_recovery_restart_last_good_target_binding_reference_binding_value,
event_log::RecoveryRestartLastGoodTargetBindingReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_restart_last_good_target_binding.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(ExecutesLifelineStatus,
"executes_lifeline_status",
{ raw("false");
}),
(ExecutesRollbackPreview,
"executes_rollback_preview",
{ raw("false");
}),
(ExecutesRollbackApply,
"executes_rollback_apply",
{ raw("false");
}),
(ExecutesDisableModule,
"executes_disable_module",
{ raw("false");
}),
(ExecutesRestartLastGood,
"executes_restart_last_good",
{ raw("false");
}),
(DisablesModule,
"disables_module",
{ raw("false");
}),
(RestartsLastGood,
"restarts_last_good",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryDisableModuleTargetBindingEventId,
"retained_recovery_disable_module_target_binding_event_id",
{ json_event_id(binding.retained_disable_module_target_binding_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(RestartLastGoodTargetId,
"restart_last_good_target_id",
{ json_str(binding.restart_last_good_target_id);
}),
(Hashes,
"hashes",
{ raw("{\"restart_last_good_target_binding_hash\": ");
json_sha256(binding.restart_last_good_target_binding_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"handler_binding_hash\": ");
json_sha256(binding.handler_binding_hash);
raw(", \"status_read_handler_hash\": ");
json_sha256(binding.status_read_handler_hash);
raw(", \"rollback_preview_authorization_hash\": ");
json_sha256(binding.rollback_preview_authorization_hash);
raw(", \"rollback_apply_authorization_hash\": ");
json_sha256(binding.rollback_apply_authorization_hash);
raw(", \"disable_module_target_binding_hash\": ");
json_sha256(binding.disable_module_target_binding_hash);
raw(", \"source_rollback_apply_denial_hash\": ");
json_sha256(binding.source_rollback_apply_denial_hash);
raw(", \"source_durable_policy_write_authority_decision_hash\": ");
json_sha256(binding.source_durable_policy_write_authority_decision_hash);
raw(", \"source_recovery_rollback_inspect_source_reference_hash\": ");
json_sha256(binding.source_recovery_rollback_inspect_source_reference_hash);
raw(", \"restart_last_good_target_projection_hash\": ");
json_sha256(binding.restart_last_good_target_projection_hash);
raw("}");
}),
}

fn emit_recovery_restart_last_good_target_binding_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryRestartLastGoodTargetBindingReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_REFERENCE_BINDING_FIELDS,
        emit_recovery_restart_last_good_target_binding_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryLoadArtifactByHashTargetBindingReferenceBindingField,
RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_REFERENCE_BINDING_FIELDS,
emit_recovery_load_artifact_by_hash_target_binding_reference_binding_value,
event_log::RecoveryLoadArtifactByHashTargetBindingReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_load_artifact_by_hash_target_binding.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(ExecutesLifelineStatus,
"executes_lifeline_status",
{ raw("false");
}),
(ExecutesRollbackPreview,
"executes_rollback_preview",
{ raw("false");
}),
(ExecutesRollbackApply,
"executes_rollback_apply",
{ raw("false");
}),
(ExecutesDisableModule,
"executes_disable_module",
{ raw("false");
}),
(ExecutesRestartLastGood,
"executes_restart_last_good",
{ raw("false");
}),
(ExecutesLoadRecoveryArtifactByHash,
"executes_load_recovery_artifact_by_hash",
{ raw("false");
}),
(DisablesModule,
"disables_module",
{ raw("false");
}),
(RestartsLastGood,
"restarts_last_good",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryRestartLastGoodTargetBindingEventId,
"retained_recovery_restart_last_good_target_binding_event_id",
{ json_event_id(binding.retained_restart_last_good_target_binding_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(LoadArtifactByHashTargetId,
"load_artifact_by_hash_target_id",
{ json_str(binding.load_artifact_by_hash_target_id);
}),
(Hashes,
"hashes",
{ raw("{\"load_artifact_by_hash_target_binding_hash\": ");
json_sha256(binding.load_artifact_by_hash_target_binding_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"handler_binding_hash\": ");
json_sha256(binding.handler_binding_hash);
raw(", \"status_read_handler_hash\": ");
json_sha256(binding.status_read_handler_hash);
raw(", \"rollback_preview_authorization_hash\": ");
json_sha256(binding.rollback_preview_authorization_hash);
raw(", \"rollback_apply_authorization_hash\": ");
json_sha256(binding.rollback_apply_authorization_hash);
raw(", \"disable_module_target_binding_hash\": ");
json_sha256(binding.disable_module_target_binding_hash);
raw(", \"restart_last_good_target_binding_hash\": ");
json_sha256(binding.restart_last_good_target_binding_hash);
raw(", \"source_rollback_apply_denial_hash\": ");
json_sha256(binding.source_rollback_apply_denial_hash);
raw(", \"source_durable_policy_write_authority_decision_hash\": ");
json_sha256(binding.source_durable_policy_write_authority_decision_hash);
raw(", \"source_recovery_rollback_inspect_source_reference_hash\": ");
json_sha256(binding.source_recovery_rollback_inspect_source_reference_hash);
raw(", \"load_artifact_by_hash_target_artifact_hash\": ");
json_sha256(binding.load_artifact_by_hash_target_artifact_hash);
raw(", \"load_artifact_by_hash_target_projection_hash\": ");
json_sha256(binding.load_artifact_by_hash_target_projection_hash);
raw("}");
}),
}

fn emit_recovery_load_artifact_by_hash_target_binding_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryLoadArtifactByHashTargetBindingReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_REFERENCE_BINDING_FIELDS,
        emit_recovery_load_artifact_by_hash_target_binding_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryMemoryWriteAuthorityReferenceBindingField,
RECOVERY_MEMORY_WRITE_AUTHORITY_REFERENCE_BINDING_FIELDS,
emit_recovery_memory_write_authority_reference_binding_value,
event_log::RecoveryMemoryWriteAuthorityReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_memory_write_authority.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(ExecutesLifelineStatus,
"executes_lifeline_status",
{ raw("false");
}),
(ExecutesRollbackPreview,
"executes_rollback_preview",
{ raw("false");
}),
(ExecutesRollbackApply,
"executes_rollback_apply",
{ raw("false");
}),
(ExecutesDisableModule,
"executes_disable_module",
{ raw("false");
}),
(ExecutesRestartLastGood,
"executes_restart_last_good",
{ raw("false");
}),
(ExecutesLoadRecoveryArtifactByHash,
"executes_load_recovery_artifact_by_hash",
{ raw("false");
}),
(DisablesModule,
"disables_module",
{ raw("false");
}),
(RestartsLastGood,
"restarts_last_good",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(WritesRecoveryMemory,
"writes_recovery_memory",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryLoadArtifactByHashTargetBindingEventId,
"retained_recovery_load_artifact_by_hash_target_binding_event_id",
{ json_event_id(binding.retained_load_artifact_by_hash_target_binding_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(RecoveryMemoryWriteAuthorityId,
"recovery_memory_write_authority_id",
{ json_str(binding.recovery_memory_write_authority_id);
}),
(Hashes,
"hashes",
{ raw("{\"recovery_memory_write_authority_hash\": ");
json_sha256(binding.recovery_memory_write_authority_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"handler_binding_hash\": ");
json_sha256(binding.handler_binding_hash);
raw(", \"status_read_handler_hash\": ");
json_sha256(binding.status_read_handler_hash);
raw(", \"rollback_preview_authorization_hash\": ");
json_sha256(binding.rollback_preview_authorization_hash);
raw(", \"rollback_apply_authorization_hash\": ");
json_sha256(binding.rollback_apply_authorization_hash);
raw(", \"disable_module_target_binding_hash\": ");
json_sha256(binding.disable_module_target_binding_hash);
raw(", \"restart_last_good_target_binding_hash\": ");
json_sha256(binding.restart_last_good_target_binding_hash);
raw(", \"load_artifact_by_hash_target_binding_hash\": ");
json_sha256(binding.load_artifact_by_hash_target_binding_hash);
raw(", \"source_rollback_apply_denial_hash\": ");
json_sha256(binding.source_rollback_apply_denial_hash);
raw(", \"source_durable_policy_write_authority_decision_hash\": ");
json_sha256(binding.source_durable_policy_write_authority_decision_hash);
raw(", \"source_recovery_rollback_inspect_source_reference_hash\": ");
json_sha256(binding.source_recovery_rollback_inspect_source_reference_hash);
raw(", \"recovery_memory_projection_hash\": ");
json_sha256(binding.recovery_memory_projection_hash);
raw("}");
}),
}

fn emit_recovery_memory_write_authority_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryMemoryWriteAuthorityReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_MEMORY_WRITE_AUTHORITY_REFERENCE_BINDING_FIELDS,
        emit_recovery_memory_write_authority_reference_binding_value,
    );
}

define_direct_binding_fields! { DurableAuditRollbackWriteAuthorityReferenceBindingField,
DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_REFERENCE_BINDING_FIELDS,
emit_durable_audit_rollback_write_authority_reference_binding_value,
event_log::DurableAuditRollbackWriteAuthorityReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.durable_audit_rollback_write_authority.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(ExecutesLifelineStatus,
"executes_lifeline_status",
{ raw("false");
}),
(ExecutesRollbackPreview,
"executes_rollback_preview",
{ raw("false");
}),
(ExecutesRollbackApply,
"executes_rollback_apply",
{ raw("false");
}),
(ExecutesDisableModule,
"executes_disable_module",
{ raw("false");
}),
(ExecutesRestartLastGood,
"executes_restart_last_good",
{ raw("false");
}),
(ExecutesLoadRecoveryArtifactByHash,
"executes_load_recovery_artifact_by_hash",
{ raw("false");
}),
(DisablesModule,
"disables_module",
{ raw("false");
}),
(RestartsLastGood,
"restarts_last_good",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(WritesRecoveryMemory,
"writes_recovery_memory",
{ raw("false");
}),
(WritesDurableAuditLog,
"writes_durable_audit_log",
{ raw("false");
}),
(WritesRollbackStore,
"writes_rollback_store",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedRecoveryMemoryWriteAuthorityEventId,
"retained_recovery_memory_write_authority_event_id",
{ json_event_id(binding.retained_recovery_memory_write_authority_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(DurableAuditRollbackWriteAuthorityId,
"durable_audit_rollback_write_authority_id",
{ json_str(binding.durable_audit_rollback_write_authority_id);
}),
(Hashes,
"hashes",
{ raw("{\"durable_audit_rollback_write_authority_hash\": ");
json_sha256(binding.durable_audit_rollback_write_authority_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"handler_binding_hash\": ");
json_sha256(binding.handler_binding_hash);
raw(", \"status_read_handler_hash\": ");
json_sha256(binding.status_read_handler_hash);
raw(", \"rollback_preview_authorization_hash\": ");
json_sha256(binding.rollback_preview_authorization_hash);
raw(", \"rollback_apply_authorization_hash\": ");
json_sha256(binding.rollback_apply_authorization_hash);
raw(", \"disable_module_target_binding_hash\": ");
json_sha256(binding.disable_module_target_binding_hash);
raw(", \"restart_last_good_target_binding_hash\": ");
json_sha256(binding.restart_last_good_target_binding_hash);
raw(", \"load_artifact_by_hash_target_binding_hash\": ");
json_sha256(binding.load_artifact_by_hash_target_binding_hash);
raw(", \"recovery_memory_write_authority_hash\": ");
json_sha256(binding.recovery_memory_write_authority_hash);
raw(", \"source_rollback_apply_denial_hash\": ");
json_sha256(binding.source_rollback_apply_denial_hash);
raw(", \"source_durable_policy_write_authority_decision_hash\": ");
json_sha256(binding.source_durable_policy_write_authority_decision_hash);
raw(", \"source_recovery_rollback_inspect_source_reference_hash\": ");
json_sha256(binding.source_recovery_rollback_inspect_source_reference_hash);
raw(", \"durable_audit_rollback_projection_hash\": ");
json_sha256(binding.durable_audit_rollback_projection_hash);
raw("}");
}),
}

fn emit_durable_audit_rollback_write_authority_reference_binding(
    kind: &str,
    binding: &event_log::DurableAuditRollbackWriteAuthorityReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_REFERENCE_BINDING_FIELDS,
        emit_durable_audit_rollback_write_authority_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryServiceInventorySideEffectBoundaryReferenceBindingField,
RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_REFERENCE_BINDING_FIELDS,
emit_recovery_service_inventory_side_effect_boundary_reference_binding_value,
event_log::RecoveryServiceInventorySideEffectBoundaryReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_service_inventory_side_effect_boundary.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(ExecutesLifelineStatus,
"executes_lifeline_status",
{ raw("false");
}),
(ExecutesRollbackPreview,
"executes_rollback_preview",
{ raw("false");
}),
(ExecutesRollbackApply,
"executes_rollback_apply",
{ raw("false");
}),
(ExecutesDisableModule,
"executes_disable_module",
{ raw("false");
}),
(ExecutesRestartLastGood,
"executes_restart_last_good",
{ raw("false");
}),
(ExecutesLoadRecoveryArtifactByHash,
"executes_load_recovery_artifact_by_hash",
{ raw("false");
}),
(DisablesModule,
"disables_module",
{ raw("false");
}),
(RestartsLastGood,
"restarts_last_good",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(WritesRecoveryMemory,
"writes_recovery_memory",
{ raw("false");
}),
(WritesDurableAuditLog,
"writes_durable_audit_log",
{ raw("false");
}),
(WritesRollbackStore,
"writes_rollback_store",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedDurableAuditRollbackWriteAuthorityEventId,
"retained_durable_audit_rollback_write_authority_event_id",
{ json_event_id(binding.retained_durable_audit_rollback_write_authority_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(ServiceInventorySideEffectBoundaryId,
"service_inventory_side_effect_boundary_id",
{ json_str(binding.service_inventory_side_effect_boundary_id);
}),
(Hashes,
"hashes",
{ raw("{\"service_inventory_side_effect_boundary_hash\": ");
json_sha256(binding.service_inventory_side_effect_boundary_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"handler_binding_hash\": ");
json_sha256(binding.handler_binding_hash);
raw(", \"status_read_handler_hash\": ");
json_sha256(binding.status_read_handler_hash);
raw(", \"rollback_preview_authorization_hash\": ");
json_sha256(binding.rollback_preview_authorization_hash);
raw(", \"rollback_apply_authorization_hash\": ");
json_sha256(binding.rollback_apply_authorization_hash);
raw(", \"disable_module_target_binding_hash\": ");
json_sha256(binding.disable_module_target_binding_hash);
raw(", \"restart_last_good_target_binding_hash\": ");
json_sha256(binding.restart_last_good_target_binding_hash);
raw(", \"load_artifact_by_hash_target_binding_hash\": ");
json_sha256(binding.load_artifact_by_hash_target_binding_hash);
raw(", \"recovery_memory_write_authority_hash\": ");
json_sha256(binding.recovery_memory_write_authority_hash);
raw(", \"durable_audit_rollback_write_authority_hash\": ");
json_sha256(binding.durable_audit_rollback_write_authority_hash);
raw(", \"source_rollback_apply_denial_hash\": ");
json_sha256(binding.source_rollback_apply_denial_hash);
raw(", \"source_durable_policy_write_authority_decision_hash\": ");
json_sha256(binding.source_durable_policy_write_authority_decision_hash);
raw(", \"source_recovery_rollback_inspect_source_reference_hash\": ");
json_sha256(binding.source_recovery_rollback_inspect_source_reference_hash);
raw(", \"service_inventory_projection_hash\": ");
json_sha256(binding.service_inventory_projection_hash);
raw("}");
}),
}

fn emit_recovery_service_inventory_side_effect_boundary_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryServiceInventorySideEffectBoundaryReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_REFERENCE_BINDING_FIELDS,
        emit_recovery_service_inventory_side_effect_boundary_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryLifelineCommandDispatchBehaviorReferenceBindingField,
RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_REFERENCE_BINDING_FIELDS,
emit_recovery_lifeline_command_dispatch_behavior_reference_binding_value,
event_log::RecoveryLifelineCommandDispatchBehaviorReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_lifeline_command_dispatch_behavior.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(ExecutesLifelineStatus,
"executes_lifeline_status",
{ raw("false");
}),
(ExecutesRollbackPreview,
"executes_rollback_preview",
{ raw("false");
}),
(ExecutesRollbackApply,
"executes_rollback_apply",
{ raw("false");
}),
(ExecutesDisableModule,
"executes_disable_module",
{ raw("false");
}),
(ExecutesRestartLastGood,
"executes_restart_last_good",
{ raw("false");
}),
(ExecutesLoadRecoveryArtifactByHash,
"executes_load_recovery_artifact_by_hash",
{ raw("false");
}),
(DisablesModule,
"disables_module",
{ raw("false");
}),
(RestartsLastGood,
"restarts_last_good",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(WritesRecoveryMemory,
"writes_recovery_memory",
{ raw("false");
}),
(WritesDurableAuditLog,
"writes_durable_audit_log",
{ raw("false");
}),
(WritesRollbackStore,
"writes_rollback_store",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedServiceInventorySideEffectBoundaryEventId,
"retained_service_inventory_side_effect_boundary_event_id",
{ json_event_id(binding.retained_service_inventory_side_effect_boundary_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(CommandDispatchBehaviorId,
"command_dispatch_behavior_id",
{ json_str(binding.command_dispatch_behavior_id);
}),
(Hashes,
"hashes",
{ raw("{\"command_dispatch_behavior_hash\": ");
json_sha256(binding.command_dispatch_behavior_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"handler_binding_hash\": ");
json_sha256(binding.handler_binding_hash);
raw(", \"status_read_handler_hash\": ");
json_sha256(binding.status_read_handler_hash);
raw(", \"rollback_preview_authorization_hash\": ");
json_sha256(binding.rollback_preview_authorization_hash);
raw(", \"rollback_apply_authorization_hash\": ");
json_sha256(binding.rollback_apply_authorization_hash);
raw(", \"disable_module_target_binding_hash\": ");
json_sha256(binding.disable_module_target_binding_hash);
raw(", \"restart_last_good_target_binding_hash\": ");
json_sha256(binding.restart_last_good_target_binding_hash);
raw(", \"load_artifact_by_hash_target_binding_hash\": ");
json_sha256(binding.load_artifact_by_hash_target_binding_hash);
raw(", \"recovery_memory_write_authority_hash\": ");
json_sha256(binding.recovery_memory_write_authority_hash);
raw(", \"durable_audit_rollback_write_authority_hash\": ");
json_sha256(binding.durable_audit_rollback_write_authority_hash);
raw(", \"service_inventory_side_effect_boundary_hash\": ");
json_sha256(binding.service_inventory_side_effect_boundary_hash);
raw(", \"source_rollback_apply_denial_hash\": ");
json_sha256(binding.source_rollback_apply_denial_hash);
raw(", \"source_durable_policy_write_authority_decision_hash\": ");
json_sha256(binding.source_durable_policy_write_authority_decision_hash);
raw(", \"source_recovery_rollback_inspect_source_reference_hash\": ");
json_sha256(binding.source_recovery_rollback_inspect_source_reference_hash);
raw(", \"command_dispatch_behavior_projection_hash\": ");
json_sha256(binding.command_dispatch_behavior_projection_hash);
raw("}");
}),
}

fn emit_recovery_lifeline_command_dispatch_behavior_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryLifelineCommandDispatchBehaviorReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_REFERENCE_BINDING_FIELDS,
        emit_recovery_lifeline_command_dispatch_behavior_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryLifelineCommandExecutorCapabilityTableReferenceBindingField,
RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_REFERENCE_BINDING_FIELDS,
emit_recovery_lifeline_command_executor_capability_table_reference_binding_value,
event_log::RecoveryLifelineCommandExecutorCapabilityTableReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_lifeline_command_executor_capability_table.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(ExecutesLifelineStatus,
"executes_lifeline_status",
{ raw("false");
}),
(ExecutesRollbackPreview,
"executes_rollback_preview",
{ raw("false");
}),
(ExecutesRollbackApply,
"executes_rollback_apply",
{ raw("false");
}),
(ExecutesDisableModule,
"executes_disable_module",
{ raw("false");
}),
(ExecutesRestartLastGood,
"executes_restart_last_good",
{ raw("false");
}),
(ExecutesLoadRecoveryArtifactByHash,
"executes_load_recovery_artifact_by_hash",
{ raw("false");
}),
(DisablesModule,
"disables_module",
{ raw("false");
}),
(RestartsLastGood,
"restarts_last_good",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(WritesRecoveryMemory,
"writes_recovery_memory",
{ raw("false");
}),
(WritesDurableAuditLog,
"writes_durable_audit_log",
{ raw("false");
}),
(WritesRollbackStore,
"writes_rollback_store",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedCommandDispatchBehaviorEventId,
"retained_command_dispatch_behavior_event_id",
{ json_event_id(binding.retained_command_dispatch_behavior_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(ExecutorCapabilityTableId,
"executor_capability_table_id",
{ json_str(binding.executor_capability_table_id);
}),
(Hashes,
"hashes",
{ raw("{\"executor_capability_table_hash\": ");
json_sha256(binding.executor_capability_table_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"handler_binding_hash\": ");
json_sha256(binding.handler_binding_hash);
raw(", \"status_read_handler_hash\": ");
json_sha256(binding.status_read_handler_hash);
raw(", \"rollback_preview_authorization_hash\": ");
json_sha256(binding.rollback_preview_authorization_hash);
raw(", \"rollback_apply_authorization_hash\": ");
json_sha256(binding.rollback_apply_authorization_hash);
raw(", \"disable_module_target_binding_hash\": ");
json_sha256(binding.disable_module_target_binding_hash);
raw(", \"restart_last_good_target_binding_hash\": ");
json_sha256(binding.restart_last_good_target_binding_hash);
raw(", \"load_artifact_by_hash_target_binding_hash\": ");
json_sha256(binding.load_artifact_by_hash_target_binding_hash);
raw(", \"recovery_memory_write_authority_hash\": ");
json_sha256(binding.recovery_memory_write_authority_hash);
raw(", \"durable_audit_rollback_write_authority_hash\": ");
json_sha256(binding.durable_audit_rollback_write_authority_hash);
raw(", \"service_inventory_side_effect_boundary_hash\": ");
json_sha256(binding.service_inventory_side_effect_boundary_hash);
raw(", \"command_dispatch_behavior_hash\": ");
json_sha256(binding.command_dispatch_behavior_hash);
raw(", \"source_rollback_apply_denial_hash\": ");
json_sha256(binding.source_rollback_apply_denial_hash);
raw(", \"source_durable_policy_write_authority_decision_hash\": ");
json_sha256(binding.source_durable_policy_write_authority_decision_hash);
raw(", \"source_recovery_rollback_inspect_source_reference_hash\": ");
json_sha256(binding.source_recovery_rollback_inspect_source_reference_hash);
raw(", \"executor_capability_projection_hash\": ");
json_sha256(binding.executor_capability_projection_hash);
raw("}");
}),
}

fn emit_recovery_lifeline_command_executor_capability_table_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryLifelineCommandExecutorCapabilityTableReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_REFERENCE_BINDING_FIELDS,
        emit_recovery_lifeline_command_executor_capability_table_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryLifelineCommandSideEffectGateReferenceBindingField,
RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_REFERENCE_BINDING_FIELDS,
emit_recovery_lifeline_command_side_effect_gate_reference_binding_value,
event_log::RecoveryLifelineCommandSideEffectGateReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_lifeline_command_side_effect_gate.v0\"");
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(ExecutesLifelineStatus,
"executes_lifeline_status",
{ raw("false");
}),
(ExecutesRollbackPreview,
"executes_rollback_preview",
{ raw("false");
}),
(ExecutesRollbackApply,
"executes_rollback_apply",
{ raw("false");
}),
(ExecutesDisableModule,
"executes_disable_module",
{ raw("false");
}),
(ExecutesRestartLastGood,
"executes_restart_last_good",
{ raw("false");
}),
(ExecutesLoadRecoveryArtifactByHash,
"executes_load_recovery_artifact_by_hash",
{ raw("false");
}),
(DisablesModule,
"disables_module",
{ raw("false");
}),
(RestartsLastGood,
"restarts_last_good",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(WritesRecoveryMemory,
"writes_recovery_memory",
{ raw("false");
}),
(WritesDurableAuditLog,
"writes_durable_audit_log",
{ raw("false");
}),
(WritesRollbackStore,
"writes_rollback_store",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedExecutorCapabilityTableEventId,
"retained_executor_capability_table_event_id",
{ json_event_id(binding.retained_executor_capability_table_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(SideEffectGateId,
"side_effect_gate_id",
{ json_str(binding.side_effect_gate_id);
}),
(Hashes,
"hashes",
{ raw("{\"side_effect_gate_hash\": ");
json_sha256(binding.side_effect_gate_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"handler_binding_hash\": ");
json_sha256(binding.handler_binding_hash);
raw(", \"status_read_handler_hash\": ");
json_sha256(binding.status_read_handler_hash);
raw(", \"rollback_preview_authorization_hash\": ");
json_sha256(binding.rollback_preview_authorization_hash);
raw(", \"rollback_apply_authorization_hash\": ");
json_sha256(binding.rollback_apply_authorization_hash);
raw(", \"disable_module_target_binding_hash\": ");
json_sha256(binding.disable_module_target_binding_hash);
raw(", \"restart_last_good_target_binding_hash\": ");
json_sha256(binding.restart_last_good_target_binding_hash);
raw(", \"load_artifact_by_hash_target_binding_hash\": ");
json_sha256(binding.load_artifact_by_hash_target_binding_hash);
raw(", \"recovery_memory_write_authority_hash\": ");
json_sha256(binding.recovery_memory_write_authority_hash);
raw(", \"durable_audit_rollback_write_authority_hash\": ");
json_sha256(binding.durable_audit_rollback_write_authority_hash);
raw(", \"service_inventory_side_effect_boundary_hash\": ");
json_sha256(binding.service_inventory_side_effect_boundary_hash);
raw(", \"command_dispatch_behavior_hash\": ");
json_sha256(binding.command_dispatch_behavior_hash);
raw(", \"executor_capability_table_hash\": ");
json_sha256(binding.executor_capability_table_hash);
raw(", \"source_rollback_apply_denial_hash\": ");
json_sha256(binding.source_rollback_apply_denial_hash);
raw(", \"source_durable_policy_write_authority_decision_hash\": ");
json_sha256(binding.source_durable_policy_write_authority_decision_hash);
raw(", \"source_recovery_rollback_inspect_source_reference_hash\": ");
json_sha256(binding.source_recovery_rollback_inspect_source_reference_hash);
raw(", \"side_effect_projection_hash\": ");
json_sha256(binding.side_effect_projection_hash);
raw("}");
}),
}

fn emit_recovery_lifeline_command_side_effect_gate_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryLifelineCommandSideEffectGateReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_REFERENCE_BINDING_FIELDS,
        emit_recovery_lifeline_command_side_effect_gate_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryLifelineCommandExecutionStageReferenceBindingField,
RECOVERY_LIFELINE_COMMAND_EXECUTION_STAGE_REFERENCE_BINDING_FIELDS,
emit_recovery_lifeline_command_execution_stage_reference_binding_value,
event_log::RecoveryLifelineCommandExecutionStageReference,
binding,
_kind;
(Schema,
"schema",
{ json_str(binding.schema);
}),
(Status,
"status",
{ raw("\"retained_hash_reference_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(StageName,
"stage_name",
{ json_str(binding.stage_name);
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(ExecutesLifelineStatus,
"executes_lifeline_status",
{ raw("false");
}),
(ExecutesRollbackPreview,
"executes_rollback_preview",
{ raw("false");
}),
(ExecutesRollbackApply,
"executes_rollback_apply",
{ raw("false");
}),
(ExecutesDisableModule,
"executes_disable_module",
{ raw("false");
}),
(ExecutesRestartLastGood,
"executes_restart_last_good",
{ raw("false");
}),
(ExecutesLoadRecoveryArtifactByHash,
"executes_load_recovery_artifact_by_hash",
{ raw("false");
}),
(DisablesModule,
"disables_module",
{ raw("false");
}),
(RestartsLastGood,
"restarts_last_good",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(CanMoveBeyondDenial,
"can_move_beyond_denial",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(WritesRecoveryMemory,
"writes_recovery_memory",
{ raw("false");
}),
(WritesDurableAuditLog,
"writes_durable_audit_log",
{ raw("false");
}),
(WritesRollbackStore,
"writes_rollback_store",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedPreviousStageEventId,
"retained_previous_stage_event_id",
{ json_event_id(binding.retained_previous_stage_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(ExecutionStageId,
"execution_stage_id",
{ json_str(binding.execution_stage_id);
}),
(Hashes,
"hashes",
{ raw("{\"execution_stage_hash\": ");
json_sha256(binding.execution_stage_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"handler_binding_hash\": ");
json_sha256(binding.handler_binding_hash);
raw(", \"status_read_handler_hash\": ");
json_sha256(binding.status_read_handler_hash);
raw(", \"rollback_preview_authorization_hash\": ");
json_sha256(binding.rollback_preview_authorization_hash);
raw(", \"rollback_apply_authorization_hash\": ");
json_sha256(binding.rollback_apply_authorization_hash);
raw(", \"disable_module_target_binding_hash\": ");
json_sha256(binding.disable_module_target_binding_hash);
raw(", \"restart_last_good_target_binding_hash\": ");
json_sha256(binding.restart_last_good_target_binding_hash);
raw(", \"load_artifact_by_hash_target_binding_hash\": ");
json_sha256(binding.load_artifact_by_hash_target_binding_hash);
raw(", \"recovery_memory_write_authority_hash\": ");
json_sha256(binding.recovery_memory_write_authority_hash);
raw(", \"durable_audit_rollback_write_authority_hash\": ");
json_sha256(binding.durable_audit_rollback_write_authority_hash);
raw(", \"service_inventory_side_effect_boundary_hash\": ");
json_sha256(binding.service_inventory_side_effect_boundary_hash);
raw(", \"command_dispatch_behavior_hash\": ");
json_sha256(binding.command_dispatch_behavior_hash);
raw(", \"executor_capability_table_hash\": ");
json_sha256(binding.executor_capability_table_hash);
raw(", \"side_effect_gate_hash\": ");
json_sha256(binding.side_effect_gate_hash);
raw(", \"source_rollback_apply_denial_hash\": ");
json_sha256(binding.source_rollback_apply_denial_hash);
raw(", \"source_durable_policy_write_authority_decision_hash\": ");
json_sha256(binding.source_durable_policy_write_authority_decision_hash);
raw(", \"source_recovery_rollback_inspect_source_reference_hash\": ");
json_sha256(binding.source_recovery_rollback_inspect_source_reference_hash);
raw(", \"execution_enablement_hash\": ");
json_sha256_option(binding.execution_enablement_hash);
raw(", \"execution_preflight_hash\": ");
json_sha256_option(binding.execution_preflight_hash);
raw(", \"execution_intent_hash\": ");
json_sha256_option(binding.execution_intent_hash);
raw(", \"execution_commit_gate_hash\": ");
json_sha256_option(binding.execution_commit_gate_hash);
raw(", \"execution_result_denial_hash\": ");
json_sha256_option(binding.execution_result_denial_hash);
raw(", \"execution_audit_denial_hash\": ");
json_sha256_option(binding.execution_audit_denial_hash);
raw(", \"execution_observation_denial_hash\": ");
json_sha256_option(binding.execution_observation_denial_hash);
raw(", \"execution_stage_projection_hash\": ");
json_sha256(binding.execution_stage_projection_hash);
raw("}");
}),
}

fn emit_recovery_lifeline_command_execution_stage_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryLifelineCommandExecutionStageReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_STAGE_REFERENCE_BINDING_FIELDS,
        emit_recovery_lifeline_command_execution_stage_reference_binding_value,
    );
}

define_direct_binding_fields! { RecoveryLifelineStatusExecutionResultReferenceBindingField,
RECOVERY_LIFELINE_STATUS_EXECUTION_RESULT_REFERENCE_BINDING_FIELDS,
emit_recovery_lifeline_status_execution_result_reference_binding_value,
event_log::RecoveryLifelineStatusExecutionResultReference,
binding,
_kind;
(Schema,
"schema",
{ raw("\"raios.recovery_lifeline_status_execution_result.v0\"");
}),
(Status,
"status",
{ raw("\"retained_read_only_result_command_still_denied\"");
}),
(Scope,
"scope",
{ raw("\"current_boot\"");
}),
(Classification,
"classification",
{ raw("\"local_only\"");
}),
(RequestedCapability,
"requested_capability",
{ raw("\"cap.recovery.command.read\"");
}),
(LoadMode,
"load_mode",
{ raw("\"recovery_only\"");
}),
(StatusExecutionReadiness,
"status_execution_readiness",
{ raw("\"available_read_only_non_authorizing\"");
}),
(ReadinessReason,
"readiness_reason",
{ raw("\"recovery_lifeline_status_read_ready_command_execution_disabled\"");
}),
(WouldExecuteLifelineStatusRead,
"would_execute_lifeline_status_read",
{ raw("true");
}),
(AcceptsRawCommandBody,
"accepts_raw_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandBody,
"accepts_lifeline_command_body",
{ raw("false");
}),
(AcceptsLifelineCommandEnvelope,
"accepts_lifeline_command_envelope",
{ raw("false");
}),
(DispatchesLifelineCommand,
"dispatches_lifeline_command",
{ raw("false");
}),
(ExecutesLifelineStatus,
"executes_lifeline_status",
{ raw("false");
}),
(CommandExecutionEnabled,
"command_execution_enabled",
{ raw("false");
}),
(AuthorizesRecoveryLoad,
"authorizes_recovery_load",
{ raw("false");
}),
(LoadsRecoveryArtifact,
"loads_recovery_artifact",
{ raw("false");
}),
(WritesRecoveryMemory,
"writes_recovery_memory",
{ raw("false");
}),
(WritesDurableAuditLog,
"writes_durable_audit_log",
{ raw("false");
}),
(WritesRollbackStore,
"writes_rollback_store",
{ raw("false");
}),
(CreatesDurableRecords,
"creates_durable_records",
{ raw("false");
}),
(InstallsRollbackPlan,
"installs_rollback_plan",
{ raw("false");
}),
(AllocatesServiceSlot,
"allocates_service_slot",
{ raw("false");
}),
(CreatesServiceInventoryRecords,
"creates_service_inventory_records",
{ raw("false");
}),
(ServiceInventoryChange,
"service_inventory_change",
{ raw("\"none\"");
}),
(LoadAttempted,
"load_attempted",
{ raw("false");
}),
(RetainedStatusReadHandlerEventId,
"retained_status_read_handler_event_id",
{ json_event_id(binding.retained_status_read_handler_event_id);
}),
(RetainedExecutionCompletionDenialEventId,
"retained_execution_completion_denial_event_id",
{ json_event_id(binding.retained_execution_completion_denial_event_id);
}),
(CommandId,
"command_id",
{ json_str(binding.command_id);
}),
(ArgumentSchema,
"argument_schema",
{ json_str(binding.argument_schema);
}),
(TargetLocator,
"target_locator",
{ json_str(binding.target_locator.as_str());
}),
(CommandDispatchBoundaryId,
"command_dispatch_boundary_id",
{ json_str(binding.command_dispatch_boundary_id);
}),
(StatusExecutionResultId,
"status_execution_result_id",
{ json_str(binding.status_execution_result_id);
}),
(Hashes,
"hashes",
{ raw("{\"status_execution_result_hash\": ");
json_sha256(binding.status_execution_result_hash);
raw(", \"argument_hash\": ");
json_sha256(binding.argument_hash);
raw(", \"command_envelope_reference_hash\": ");
json_sha256(binding.command_envelope_reference_hash);
raw(", \"command_body_canonicalization_hash\": ");
json_sha256(binding.command_body_canonicalization_hash);
raw(", \"handler_binding_hash\": ");
json_sha256(binding.handler_binding_hash);
raw(", \"status_read_handler_hash\": ");
json_sha256(binding.status_read_handler_hash);
raw(", \"status_read_projection_hash\": ");
json_sha256(binding.status_read_projection_hash);
raw(", \"command_dispatch_behavior_hash\": ");
json_sha256(binding.command_dispatch_behavior_hash);
raw(", \"executor_capability_table_hash\": ");
json_sha256(binding.executor_capability_table_hash);
raw(", \"side_effect_gate_hash\": ");
json_sha256(binding.side_effect_gate_hash);
raw(", \"execution_enablement_hash\": ");
json_sha256(binding.execution_enablement_hash);
raw(", \"execution_preflight_hash\": ");
json_sha256(binding.execution_preflight_hash);
raw(", \"execution_intent_hash\": ");
json_sha256(binding.execution_intent_hash);
raw(", \"execution_commit_gate_hash\": ");
json_sha256(binding.execution_commit_gate_hash);
raw(", \"execution_result_denial_hash\": ");
json_sha256(binding.execution_result_denial_hash);
raw(", \"execution_audit_denial_hash\": ");
json_sha256(binding.execution_audit_denial_hash);
raw(", \"execution_observation_denial_hash\": ");
json_sha256(binding.execution_observation_denial_hash);
raw(", \"execution_completion_denial_hash\": ");
json_sha256(binding.execution_completion_denial_hash);
raw("}");
}),
}

fn emit_recovery_lifeline_status_execution_result_reference_binding(
    kind: &str,
    binding: &event_log::RecoveryLifelineStatusExecutionResultReference,
) {
    emit_binding_object_direct(
        binding,
        kind,
        RECOVERY_LIFELINE_STATUS_EXECUTION_RESULT_REFERENCE_BINDING_FIELDS,
        emit_recovery_lifeline_status_execution_result_reference_binding_value,
    );
}

fn emit_provider_trust_verifier_metadata(metadata: provider_trust::ProviderTrustVerifierMetadata) {
    raw("{\"schema\": ");
    json_str(metadata.schema);
    raw(", \"id\": ");
    json_str(metadata.id);
    raw(", \"host\": ");
    json_str(metadata.host);
    raw(", \"port\": ");
    json_str(metadata.port);
    raw(", \"transport\": ");
    json_str(metadata.transport);
    raw(", \"hostname_policy\": ");
    json_str(metadata.hostname_policy);
    raw(", \"pin_policy\": ");
    json_str(metadata.pin_policy);
    raw(", \"chain_policy\": ");
    json_str(metadata.chain_policy);
    raw(", \"time_policy\": ");
    json_str(metadata.time_policy);
    raw(", \"certificate_verify_policy\": ");
    json_str(metadata.certificate_verify_policy);
    raw("}");
}

fn emit_provider_trust_verifier_decision(decision: provider_trust::ProviderTrustVerifierDecision) {
    raw("{\"schema\": ");
    json_str(decision.schema);
    raw(", \"verifier_id\": ");
    json_str(decision.verifier_id);
    raw(", \"stage\": ");
    json_str(decision.stage);
    raw(", \"outcome\": ");
    json_str(decision.outcome);
    raw(", \"reason\": ");
    json_str(decision.reason);
    raw("}");
}

fn emit_memory_record(
    id: &'static str,
    kind: &'static str,
    authority: &'static str,
    classification: &'static str,
    summary: &'static str,
    source: &'static str,
    comma: bool,
) {
    indent(8);
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
    raw(", \"source\": ");
    json_str(source);
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_memory_candidate(
    id: &'static str,
    kind: &'static str,
    classification: &'static str,
    summary: &'static str,
    comma: bool,
) {
    indent(8);
    raw("{\"id\": ");
    json_str(id);
    raw(", \"kind\": ");
    json_str(kind);
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

fn emit_trace_record(
    id: &'static str,
    source_method: &'static str,
    source: &'static str,
    comma: bool,
) {
    indent(8);
    raw("{\"id\": ");
    json_str(id);
    raw(", \"found\": true, \"source_method\": ");
    json_str(source_method);
    raw(", \"source\": ");
    json_str(source);
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_single_trace_record(id: &str) {
    if method_eq(id, "mem.fact.identity.stage0") {
        emit_trace_record(
            "mem.fact.identity.stage0",
            "system.describe",
            "seed-kernel/src/agent_protocol.rs",
            false,
        );
    } else if method_eq(id, "snapshot.current") {
        emit_trace_record(
            "snapshot.current",
            "system.snapshot",
            "seed-kernel/src/agent_protocol.rs",
            false,
        );
    } else if method_eq(id, "snapshot.current.provider_minimal") {
        emit_trace_record(
            "snapshot.current.provider_minimal",
            "memory.context provider_minimal",
            "seed-kernel/src/agent_protocol.rs",
            false,
        );
    } else if method_eq(id, "capabilities.current_boot") {
        emit_trace_record(
            "capabilities.current_boot",
            "system.capabilities",
            "seed-kernel/src/agent_protocol.rs",
            false,
        );
    } else if method_eq(id, "service.inventory.current") {
        emit_trace_record(
            "service.inventory.current",
            "service.inventory",
            "seed-kernel/src/service_inventory.rs",
            false,
        );
    } else if method_eq(id, "problem.list.current") {
        emit_trace_record(
            "problem.list.current",
            "problem.list",
            "seed-kernel/src/agent_protocol.rs",
            false,
        );
    } else if method_eq(id, "boot_log.summary.current") {
        emit_trace_record(
            "boot_log.summary.current",
            "system.boot_log",
            "seed-kernel/src/serial.rs",
            false,
        );
    } else if method_eq(id, "adr.0001") {
        emit_trace_record(
            "adr.0001",
            "decision",
            "docs/architecture-decisions/0001-raios-agent-protocol.md",
            false,
        );
    } else if method_eq(id, "adr.0004") {
        emit_trace_record(
            "adr.0004",
            "decision",
            "docs/architecture-decisions/0004-system-memory-and-agent-context.md",
            false,
        );
    } else {
        indent(8);
        raw("{\"id\": ");
        json_str(id);
        raw(", \"found\": false, \"reason\": \"record id is not in the current_boot memory index\"}");
        crlf();
    }
}
