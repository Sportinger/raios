use crate::{
    agent_protocol_memory::{
        canonical_memory_mutation_method, emit_memory_capability_denied, emit_memory_context,
        emit_memory_profile, emit_memory_query, emit_memory_trace, emit_recent_events,
        memory_mutation_method,
    },
    agent_protocol_module_approval::{
        emit_module_approval_diagnostic, emit_module_approval_diagnostic_selftest,
        module_approval_diagnostic_method, module_approval_diagnostic_selftest_method,
    },
    agent_protocol_module_attestation::{
        emit_module_attestation_diagnostic, emit_module_attestation_diagnostic_selftest,
        module_attestation_diagnostic_method, module_attestation_diagnostic_selftest_method,
    },
    agent_protocol_module_audit::{
        emit_module_audit_rollback_diagnostic, emit_module_audit_rollback_diagnostic_selftest,
        module_audit_rollback_diagnostic_method, module_audit_rollback_diagnostic_selftest_method,
    },
    agent_protocol_module_grant::{
        emit_module_grant_diagnostic, emit_module_grant_diagnostic_selftest,
        module_grant_diagnostic_method, module_grant_diagnostic_selftest_method,
    },
    agent_protocol_module_load_gate::{
        emit_module_load_ephemeral_denied, emit_module_load_gate_approval_selftest,
        emit_module_load_gate_artifact_selftest, emit_module_load_gate_attestation_selftest,
        emit_module_load_gate_audit_rollback_selftest, emit_module_load_gate_manifest_selftest,
        emit_module_load_gate_retained_selftest, emit_module_load_gate_service_slot_selftest,
        emit_module_load_gate_vm_report_selftest, module_load_gate_approval_selftest_method,
        module_load_gate_artifact_selftest_method, module_load_gate_attestation_selftest_method,
        module_load_gate_audit_rollback_selftest_method, module_load_gate_manifest_selftest_method,
        module_load_gate_retained_selftest_method, module_load_gate_service_slot_selftest_method,
        module_load_gate_vm_report_selftest_method,
    },
    agent_protocol_module_reference::{
        emit_module_artifact_diagnostic, emit_module_artifact_diagnostic_selftest,
        emit_module_manifest_diagnostic, emit_module_manifest_diagnostic_selftest,
        emit_module_vm_report_diagnostic, emit_module_vm_report_diagnostic_selftest,
        module_artifact_diagnostic_method, module_artifact_diagnostic_selftest_method,
        module_manifest_diagnostic_method, module_manifest_diagnostic_selftest_method,
        module_vm_report_diagnostic_method, module_vm_report_diagnostic_selftest_method,
    },
    agent_protocol_module_service_slot::{
        emit_module_service_slot_diagnostic, emit_module_service_slot_diagnostic_selftest,
        module_service_slot_diagnostic_method, module_service_slot_diagnostic_selftest_method,
    },
    agent_protocol_module_write_boundary::{
        emit_module_audit_rollback_append_contract,
        emit_module_audit_rollback_append_contract_selftest,
        emit_module_audit_rollback_append_engine,
        emit_module_audit_rollback_append_engine_selftest,
        emit_module_audit_rollback_append_intent,
        emit_module_audit_rollback_append_intent_selftest,
        emit_module_audit_rollback_append_payload_hash,
        emit_module_audit_rollback_append_payload_hash_selftest,
        emit_module_audit_rollback_availability, emit_module_audit_rollback_availability_selftest,
        emit_module_audit_rollback_storage_layout,
        emit_module_audit_rollback_storage_layout_selftest,
        emit_module_audit_rollback_write_boundary,
        emit_module_audit_rollback_write_boundary_selftest,
        emit_module_audit_rollback_write_policy, emit_module_audit_rollback_write_policy_selftest,
        module_audit_rollback_append_contract_method,
        module_audit_rollback_append_contract_selftest_method,
        module_audit_rollback_append_engine_method,
        module_audit_rollback_append_engine_selftest_method,
        module_audit_rollback_append_intent_method,
        module_audit_rollback_append_intent_selftest_method,
        module_audit_rollback_append_payload_hash_method,
        module_audit_rollback_append_payload_hash_selftest_method,
        module_audit_rollback_availability_method,
        module_audit_rollback_availability_selftest_method,
        module_audit_rollback_storage_layout_method,
        module_audit_rollback_storage_layout_selftest_method,
        module_audit_rollback_write_boundary_method,
        module_audit_rollback_write_boundary_selftest_method,
        module_audit_rollback_write_policy_method,
        module_audit_rollback_write_policy_selftest_method,
    },
    agent_protocol_policy::{
        canonical_denied_method, canonical_module_load_ephemeral_method, denied_method,
        emit_capability_denied, module_load_ephemeral_method, record_denial, record_read,
    },
    agent_protocol_provider::{
        emit_provider_context_export_denied, emit_provider_context_gate,
        emit_provider_context_gate_selftest, emit_provider_context_injection_gate,
        emit_provider_context_injection_gate_selftest, provider_context_export_method,
        provider_context_gate_method, provider_context_gate_selftest_method,
        provider_context_injection_gate_method, provider_context_injection_gate_selftest_method,
    },
    agent_protocol_recovery::{
        canonical_recovery_artifact_load_method, emit_recovery_artifact_identity_diagnostic,
        emit_recovery_artifact_identity_diagnostic_selftest, emit_recovery_artifact_load_binding,
        emit_recovery_artifact_load_binding_selftest, emit_recovery_artifact_load_denied,
        emit_recovery_artifact_loader_diagnostic,
        emit_recovery_artifact_loader_diagnostic_selftest,
        emit_recovery_artifact_local_approval_diagnostic,
        emit_recovery_artifact_local_approval_diagnostic_selftest,
        emit_recovery_artifact_rollback_evidence_diagnostic,
        emit_recovery_artifact_rollback_evidence_diagnostic_selftest,
        emit_recovery_artifact_trust_diagnostic, emit_recovery_artifact_trust_diagnostic_selftest,
        emit_recovery_artifact_vm_test_diagnostic,
        emit_recovery_artifact_vm_test_diagnostic_selftest,
        emit_recovery_lifeline_protocol_diagnostic,
        emit_recovery_lifeline_protocol_diagnostic_selftest,
        emit_recovery_lifeline_request_diagnostic,
        emit_recovery_lifeline_request_diagnostic_selftest,
        recovery_artifact_identity_diagnostic_method,
        recovery_artifact_identity_diagnostic_selftest_method,
        recovery_artifact_load_binding_method, recovery_artifact_load_binding_selftest_method,
        recovery_artifact_load_method, recovery_artifact_loader_diagnostic_method,
        recovery_artifact_loader_diagnostic_selftest_method,
        recovery_artifact_local_approval_diagnostic_method,
        recovery_artifact_local_approval_diagnostic_selftest_method,
        recovery_artifact_rollback_evidence_diagnostic_method,
        recovery_artifact_rollback_evidence_diagnostic_selftest_method,
        recovery_artifact_trust_diagnostic_method,
        recovery_artifact_trust_diagnostic_selftest_method,
        recovery_artifact_vm_test_diagnostic_method,
        recovery_artifact_vm_test_diagnostic_selftest_method,
        recovery_lifeline_protocol_diagnostic_method,
        recovery_lifeline_protocol_diagnostic_selftest_method,
        recovery_lifeline_request_diagnostic_method,
        recovery_lifeline_request_diagnostic_selftest_method,
    },
    agent_protocol_support::{method_eq, method_head_eq},
    agent_protocol_system::{
        emit_boot_log, emit_capabilities, emit_describe, emit_device_graph, emit_problem_list,
        emit_service_inventory, emit_snapshot,
    },
    event_log, ui,
};

pub(crate) use crate::agent_protocol_provider::provider_minimal_context_evidence_for_runtime;

pub enum DispatchOutcome {
    Response(&'static str),
    Denied(&'static str),
    Unknown,
}

pub fn dispatch(method: &str, runtime: ui::RuntimeStatus) -> DispatchOutcome {
    let method = method.trim();
    if method.is_empty() {
        return DispatchOutcome::Unknown;
    }

    if method_eq(method, "system.describe") || method_eq(method, "describe") {
        record_read("system.describe");
        emit_describe();
        return DispatchOutcome::Response("system.describe");
    }
    if method_eq(method, "system.snapshot") || method_eq(method, "snapshot") {
        record_read("system.snapshot");
        emit_snapshot(runtime);
        return DispatchOutcome::Response("system.snapshot");
    }
    if method_eq(method, "system.capabilities")
        || method_eq(method, "capabilities")
        || method_eq(method, "caps")
    {
        record_read("system.capabilities");
        emit_capabilities();
        return DispatchOutcome::Response("system.capabilities");
    }
    if method_eq(method, "system.boot_log")
        || method_eq(method, "system.bootlog")
        || method_eq(method, "bootlog")
    {
        record_read("system.boot_log");
        emit_boot_log();
        return DispatchOutcome::Response("system.boot_log");
    }
    if method_eq(method, "device.graph") || method_eq(method, "devicegraph") {
        record_read("device.graph");
        emit_device_graph(runtime);
        return DispatchOutcome::Response("device.graph");
    }
    if method_eq(method, "problem.list") || method_eq(method, "problems") {
        record_read("problem.list");
        emit_problem_list(runtime);
        return DispatchOutcome::Response("problem.list");
    }
    if method_eq(method, "service.inventory") || method_eq(method, "services") {
        record_read("service.inventory");
        emit_service_inventory(runtime);
        return DispatchOutcome::Response("service.inventory");
    }
    if method_eq(method, "memory.profile") || method_eq(method, "memprofile") {
        record_read("memory.profile");
        emit_memory_profile();
        return DispatchOutcome::Response("memory.profile");
    }
    if method_head_eq(method, "memory.context") || method_head_eq(method, "memctx") {
        let event_id = record_read("memory.context");
        emit_memory_context(runtime, method, event_id);
        return DispatchOutcome::Response("memory.context");
    }
    if method_head_eq(method, "memory.query") || method_head_eq(method, "memquery") {
        record_read("memory.query");
        emit_memory_query();
        return DispatchOutcome::Response("memory.query");
    }
    if method_head_eq(method, "memory.trace") || method_head_eq(method, "memtrace") {
        record_read("memory.trace");
        emit_memory_trace(method);
        return DispatchOutcome::Response("memory.trace");
    }
    if method_head_eq(method, "memory.recent_events")
        || method_head_eq(method, "audit.events")
        || method_head_eq(method, "events")
    {
        record_read("memory.recent_events");
        emit_recent_events(method);
        return DispatchOutcome::Response("memory.recent_events");
    }
    if provider_context_gate_method(method) {
        record_read("provider.context_gate");
        emit_provider_context_gate(runtime, method);
        return DispatchOutcome::Response("provider.context_gate");
    }
    if provider_context_gate_selftest_method(method) {
        record_read("provider.context_gate_selftest");
        emit_provider_context_gate_selftest(runtime, method);
        return DispatchOutcome::Response("provider.context_gate_selftest");
    }
    if provider_context_injection_gate_method(method) {
        record_read("provider.context_injection_gate");
        emit_provider_context_injection_gate(runtime, method);
        return DispatchOutcome::Response("provider.context_injection_gate");
    }
    if provider_context_injection_gate_selftest_method(method) {
        record_read("provider.context_injection_gate_selftest");
        emit_provider_context_injection_gate_selftest(runtime, method);
        return DispatchOutcome::Response("provider.context_injection_gate_selftest");
    }
    if module_manifest_diagnostic_method(method) {
        record_read("module.manifest_diagnostic");
        emit_module_manifest_diagnostic(method);
        return DispatchOutcome::Response("module.manifest_diagnostic");
    }
    if module_manifest_diagnostic_selftest_method(method) {
        record_read("module.manifest_diagnostic_selftest");
        emit_module_manifest_diagnostic_selftest();
        return DispatchOutcome::Response("module.manifest_diagnostic_selftest");
    }
    if module_artifact_diagnostic_method(method) {
        record_read("module.artifact_diagnostic");
        emit_module_artifact_diagnostic(method);
        return DispatchOutcome::Response("module.artifact_diagnostic");
    }
    if module_artifact_diagnostic_selftest_method(method) {
        record_read("module.artifact_diagnostic_selftest");
        emit_module_artifact_diagnostic_selftest();
        return DispatchOutcome::Response("module.artifact_diagnostic_selftest");
    }
    if module_vm_report_diagnostic_method(method) {
        record_read("module.vm_report_diagnostic");
        emit_module_vm_report_diagnostic(method);
        return DispatchOutcome::Response("module.vm_report_diagnostic");
    }
    if module_vm_report_diagnostic_selftest_method(method) {
        record_read("module.vm_report_diagnostic_selftest");
        emit_module_vm_report_diagnostic_selftest();
        return DispatchOutcome::Response("module.vm_report_diagnostic_selftest");
    }
    if module_attestation_diagnostic_method(method) {
        record_read("module.attestation_diagnostic");
        emit_module_attestation_diagnostic(method);
        return DispatchOutcome::Response("module.attestation_diagnostic");
    }
    if module_attestation_diagnostic_selftest_method(method) {
        record_read("module.attestation_diagnostic_selftest");
        emit_module_attestation_diagnostic_selftest();
        return DispatchOutcome::Response("module.attestation_diagnostic_selftest");
    }
    if module_approval_diagnostic_method(method) {
        record_read("module.approval_diagnostic");
        emit_module_approval_diagnostic(method);
        return DispatchOutcome::Response("module.approval_diagnostic");
    }
    if module_approval_diagnostic_selftest_method(method) {
        record_read("module.approval_diagnostic_selftest");
        emit_module_approval_diagnostic_selftest();
        return DispatchOutcome::Response("module.approval_diagnostic_selftest");
    }
    if module_grant_diagnostic_method(method) {
        record_read("module.grant_diagnostic");
        emit_module_grant_diagnostic(method);
        return DispatchOutcome::Response("module.grant_diagnostic");
    }
    if module_grant_diagnostic_selftest_method(method) {
        record_read("module.grant_diagnostic_selftest");
        emit_module_grant_diagnostic_selftest();
        return DispatchOutcome::Response("module.grant_diagnostic_selftest");
    }
    if module_audit_rollback_diagnostic_method(method) {
        record_read("module.audit_rollback_diagnostic");
        emit_module_audit_rollback_diagnostic(method);
        return DispatchOutcome::Response("module.audit_rollback_diagnostic");
    }
    if module_audit_rollback_diagnostic_selftest_method(method) {
        record_read("module.audit_rollback_diagnostic_selftest");
        emit_module_audit_rollback_diagnostic_selftest();
        return DispatchOutcome::Response("module.audit_rollback_diagnostic_selftest");
    }
    if module_service_slot_diagnostic_method(method) {
        record_read("module.service_slot_diagnostic");
        emit_module_service_slot_diagnostic(method);
        return DispatchOutcome::Response("module.service_slot_diagnostic");
    }
    if module_service_slot_diagnostic_selftest_method(method) {
        record_read("module.service_slot_diagnostic_selftest");
        emit_module_service_slot_diagnostic_selftest();
        return DispatchOutcome::Response("module.service_slot_diagnostic_selftest");
    }
    if module_audit_rollback_availability_method(method) {
        record_read("module.audit_rollback_availability");
        emit_module_audit_rollback_availability();
        return DispatchOutcome::Response("module.audit_rollback_availability");
    }
    if module_audit_rollback_availability_selftest_method(method) {
        record_read("module.audit_rollback_availability_selftest");
        emit_module_audit_rollback_availability_selftest();
        return DispatchOutcome::Response("module.audit_rollback_availability_selftest");
    }
    if module_audit_rollback_write_policy_method(method) {
        record_read("module.audit_rollback_write_policy");
        emit_module_audit_rollback_write_policy();
        return DispatchOutcome::Response("module.audit_rollback_write_policy");
    }
    if module_audit_rollback_write_policy_selftest_method(method) {
        record_read("module.audit_rollback_write_policy_selftest");
        emit_module_audit_rollback_write_policy_selftest();
        return DispatchOutcome::Response("module.audit_rollback_write_policy_selftest");
    }
    if module_audit_rollback_storage_layout_method(method) {
        record_read("module.audit_rollback_storage_layout");
        emit_module_audit_rollback_storage_layout();
        return DispatchOutcome::Response("module.audit_rollback_storage_layout");
    }
    if module_audit_rollback_storage_layout_selftest_method(method) {
        record_read("module.audit_rollback_storage_layout_selftest");
        emit_module_audit_rollback_storage_layout_selftest();
        return DispatchOutcome::Response("module.audit_rollback_storage_layout_selftest");
    }
    if module_audit_rollback_append_engine_method(method) {
        record_read("module.audit_rollback_append_engine");
        emit_module_audit_rollback_append_engine();
        return DispatchOutcome::Response("module.audit_rollback_append_engine");
    }
    if module_audit_rollback_append_engine_selftest_method(method) {
        record_read("module.audit_rollback_append_engine_selftest");
        emit_module_audit_rollback_append_engine_selftest();
        return DispatchOutcome::Response("module.audit_rollback_append_engine_selftest");
    }
    if module_audit_rollback_append_contract_method(method) {
        record_read("module.audit_rollback_append_contract");
        emit_module_audit_rollback_append_contract();
        return DispatchOutcome::Response("module.audit_rollback_append_contract");
    }
    if module_audit_rollback_append_contract_selftest_method(method) {
        record_read("module.audit_rollback_append_contract_selftest");
        emit_module_audit_rollback_append_contract_selftest();
        return DispatchOutcome::Response("module.audit_rollback_append_contract_selftest");
    }
    if module_audit_rollback_append_payload_hash_method(method) {
        record_read("module.audit_rollback_append_payload_hash");
        emit_module_audit_rollback_append_payload_hash();
        return DispatchOutcome::Response("module.audit_rollback_append_payload_hash");
    }
    if module_audit_rollback_append_payload_hash_selftest_method(method) {
        record_read("module.audit_rollback_append_payload_hash_selftest");
        emit_module_audit_rollback_append_payload_hash_selftest();
        return DispatchOutcome::Response("module.audit_rollback_append_payload_hash_selftest");
    }
    if module_audit_rollback_append_intent_method(method) {
        record_read("module.audit_rollback_append_intent");
        emit_module_audit_rollback_append_intent();
        return DispatchOutcome::Response("module.audit_rollback_append_intent");
    }
    if module_audit_rollback_append_intent_selftest_method(method) {
        record_read("module.audit_rollback_append_intent_selftest");
        emit_module_audit_rollback_append_intent_selftest();
        return DispatchOutcome::Response("module.audit_rollback_append_intent_selftest");
    }
    if module_audit_rollback_write_boundary_method(method) {
        record_read("module.audit_rollback_write_boundary");
        emit_module_audit_rollback_write_boundary();
        return DispatchOutcome::Response("module.audit_rollback_write_boundary");
    }
    if module_audit_rollback_write_boundary_selftest_method(method) {
        record_read("module.audit_rollback_write_boundary_selftest");
        emit_module_audit_rollback_write_boundary_selftest();
        return DispatchOutcome::Response("module.audit_rollback_write_boundary_selftest");
    }
    if module_load_gate_manifest_selftest_method(method) {
        record_read("module.load_gate_manifest_selftest");
        emit_module_load_gate_manifest_selftest();
        return DispatchOutcome::Response("module.load_gate_manifest_selftest");
    }
    if module_load_gate_artifact_selftest_method(method) {
        record_read("module.load_gate_artifact_selftest");
        emit_module_load_gate_artifact_selftest();
        return DispatchOutcome::Response("module.load_gate_artifact_selftest");
    }
    if module_load_gate_vm_report_selftest_method(method) {
        record_read("module.load_gate_vm_report_selftest");
        emit_module_load_gate_vm_report_selftest();
        return DispatchOutcome::Response("module.load_gate_vm_report_selftest");
    }
    if module_load_gate_attestation_selftest_method(method) {
        record_read("module.load_gate_attestation_selftest");
        emit_module_load_gate_attestation_selftest();
        return DispatchOutcome::Response("module.load_gate_attestation_selftest");
    }
    if module_load_gate_approval_selftest_method(method) {
        record_read("module.load_gate_approval_selftest");
        emit_module_load_gate_approval_selftest();
        return DispatchOutcome::Response("module.load_gate_approval_selftest");
    }
    if module_load_gate_retained_selftest_method(method) {
        record_read("module.load_gate_retained_selftest");
        emit_module_load_gate_retained_selftest();
        return DispatchOutcome::Response("module.load_gate_retained_selftest");
    }
    if module_load_gate_audit_rollback_selftest_method(method) {
        record_read("module.load_gate_audit_rollback_selftest");
        emit_module_load_gate_audit_rollback_selftest();
        return DispatchOutcome::Response("module.load_gate_audit_rollback_selftest");
    }
    if module_load_gate_service_slot_selftest_method(method) {
        record_read("module.load_gate_service_slot_selftest");
        emit_module_load_gate_service_slot_selftest();
        return DispatchOutcome::Response("module.load_gate_service_slot_selftest");
    }
    if recovery_artifact_identity_diagnostic_method(method) {
        record_read("recovery.identity_diagnostic");
        emit_recovery_artifact_identity_diagnostic(method);
        return DispatchOutcome::Response("recovery.identity_diagnostic");
    }
    if recovery_artifact_identity_diagnostic_selftest_method(method) {
        record_read("recovery.identity_diagnostic_selftest");
        emit_recovery_artifact_identity_diagnostic_selftest();
        return DispatchOutcome::Response("recovery.identity_diagnostic_selftest");
    }
    if recovery_artifact_trust_diagnostic_method(method) {
        record_read("recovery.trust_diagnostic");
        emit_recovery_artifact_trust_diagnostic(method);
        return DispatchOutcome::Response("recovery.trust_diagnostic");
    }
    if recovery_artifact_trust_diagnostic_selftest_method(method) {
        record_read("recovery.trust_diagnostic_selftest");
        emit_recovery_artifact_trust_diagnostic_selftest();
        return DispatchOutcome::Response("recovery.trust_diagnostic_selftest");
    }
    if recovery_artifact_vm_test_diagnostic_method(method) {
        record_read("recovery.vm_test_diagnostic");
        emit_recovery_artifact_vm_test_diagnostic(method);
        return DispatchOutcome::Response("recovery.vm_test_diagnostic");
    }
    if recovery_artifact_vm_test_diagnostic_selftest_method(method) {
        record_read("recovery.vm_test_diagnostic_selftest");
        emit_recovery_artifact_vm_test_diagnostic_selftest();
        return DispatchOutcome::Response("recovery.vm_test_diagnostic_selftest");
    }
    if recovery_artifact_local_approval_diagnostic_method(method) {
        record_read("recovery.local_approval_diagnostic");
        emit_recovery_artifact_local_approval_diagnostic(method);
        return DispatchOutcome::Response("recovery.local_approval_diagnostic");
    }
    if recovery_artifact_local_approval_diagnostic_selftest_method(method) {
        record_read("recovery.local_approval_diagnostic_selftest");
        emit_recovery_artifact_local_approval_diagnostic_selftest();
        return DispatchOutcome::Response("recovery.local_approval_diagnostic_selftest");
    }
    if recovery_artifact_loader_diagnostic_method(method) {
        record_read("recovery.loader_diagnostic");
        emit_recovery_artifact_loader_diagnostic(method);
        return DispatchOutcome::Response("recovery.loader_diagnostic");
    }
    if recovery_artifact_loader_diagnostic_selftest_method(method) {
        record_read("recovery.loader_diagnostic_selftest");
        emit_recovery_artifact_loader_diagnostic_selftest();
        return DispatchOutcome::Response("recovery.loader_diagnostic_selftest");
    }
    if recovery_artifact_rollback_evidence_diagnostic_method(method) {
        record_read("recovery.rollback_evidence_diagnostic");
        emit_recovery_artifact_rollback_evidence_diagnostic(method);
        return DispatchOutcome::Response("recovery.rollback_evidence_diagnostic");
    }
    if recovery_artifact_rollback_evidence_diagnostic_selftest_method(method) {
        record_read("recovery.rollback_evidence_diagnostic_selftest");
        emit_recovery_artifact_rollback_evidence_diagnostic_selftest();
        return DispatchOutcome::Response("recovery.rollback_evidence_diagnostic_selftest");
    }
    if recovery_lifeline_request_diagnostic_method(method) {
        record_read("recovery.lifeline_request_diagnostic");
        emit_recovery_lifeline_request_diagnostic(method);
        return DispatchOutcome::Response("recovery.lifeline_request_diagnostic");
    }
    if recovery_lifeline_request_diagnostic_selftest_method(method) {
        record_read("recovery.lifeline_request_diagnostic_selftest");
        emit_recovery_lifeline_request_diagnostic_selftest();
        return DispatchOutcome::Response("recovery.lifeline_request_diagnostic_selftest");
    }
    if recovery_lifeline_protocol_diagnostic_method(method) {
        record_read("recovery.lifeline_protocol_diagnostic");
        emit_recovery_lifeline_protocol_diagnostic();
        return DispatchOutcome::Response("recovery.lifeline_protocol_diagnostic");
    }
    if recovery_lifeline_protocol_diagnostic_selftest_method(method) {
        record_read("recovery.lifeline_protocol_diagnostic_selftest");
        emit_recovery_lifeline_protocol_diagnostic_selftest();
        return DispatchOutcome::Response("recovery.lifeline_protocol_diagnostic_selftest");
    }
    if recovery_artifact_load_binding_method(method) {
        record_read("recovery.load_binding");
        emit_recovery_artifact_load_binding();
        return DispatchOutcome::Response("recovery.load_binding");
    }
    if recovery_artifact_load_binding_selftest_method(method) {
        record_read("recovery.load_binding_selftest");
        emit_recovery_artifact_load_binding_selftest();
        return DispatchOutcome::Response("recovery.load_binding_selftest");
    }

    if provider_context_export_method(method) {
        let event_id = record_denial("provider.context_export");
        emit_provider_context_export_denied(runtime, method, event_id);
        return DispatchOutcome::Denied("provider.context_export");
    }

    if module_load_ephemeral_method(method) {
        let method = canonical_module_load_ephemeral_method(method);
        let (event_id, gate_binding) = event_log::record_module_load_ephemeral_denied(method);
        emit_module_load_ephemeral_denied(method, event_id, gate_binding);
        return DispatchOutcome::Denied(method);
    }

    if recovery_artifact_load_method(method) {
        let method = canonical_recovery_artifact_load_method(method);
        let event_id = event_log::record_recovery_artifact_load_denied(method);
        emit_recovery_artifact_load_denied(method, event_id);
        return DispatchOutcome::Denied(method);
    }

    if memory_mutation_method(method) {
        let method = canonical_memory_mutation_method(method);
        let event_id = record_denial(method);
        emit_memory_capability_denied(method, event_id);
        return DispatchOutcome::Denied(method);
    }

    if denied_method(method) {
        let method = canonical_denied_method(method);
        let event_id = record_denial(method);
        emit_capability_denied(method, event_id);
        return DispatchOutcome::Denied(method);
    }

    DispatchOutcome::Unknown
}
