#[path = "artifact_store.rs"]
pub(crate) mod artifact_store;
#[path = "boot_control.rs"]
mod boot_control;
#[path = "durable_store.rs"]
pub(crate) mod durable_store;
#[path = "recovery_lifeline.rs"]
mod recovery_lifeline;
#[path = "repromotion.rs"]
mod repromotion;

use crate::{
    agent_protocol_memory::{
        emit_memory_capability_denied, emit_memory_context, emit_memory_profile, emit_memory_query,
        emit_memory_trace, emit_recent_events,
    },
    agent_protocol_module_approval::{
        emit_module_approval_diagnostic, emit_module_approval_diagnostic_selftest,
    },
    agent_protocol_module_attestation::{
        emit_module_attestation_diagnostic, emit_module_attestation_diagnostic_selftest,
    },
    agent_protocol_module_audit::{
        emit_module_audit_rollback_diagnostic, emit_module_audit_rollback_diagnostic_selftest,
    },
    agent_protocol_module_grant::{
        emit_module_grant_diagnostic, emit_module_grant_diagnostic_selftest,
    },
    agent_protocol_module_load_gate::{
        emit_module_load_ephemeral_denied, emit_module_load_gate_approval_selftest,
        emit_module_load_gate_artifact_selftest, emit_module_load_gate_attestation_selftest,
        emit_module_load_gate_audit_rollback_selftest,
        emit_module_load_gate_loader_runtime_selftest, emit_module_load_gate_manifest_selftest,
        emit_module_load_gate_retained_selftest, emit_module_load_gate_service_slot_selftest,
        emit_module_load_gate_vm_report_selftest,
    },
    agent_protocol_module_loader_artifact_hash_binding::{
        emit_module_loader_artifact_hash_binding, emit_module_loader_artifact_hash_binding_selftest,
    },
    agent_protocol_module_loader_fact::{
        emit_module_loader_fact, emit_module_loader_fact_selftest,
    },
    agent_protocol_module_loader_identity::{
        emit_module_loader_identity, emit_module_loader_identity_selftest,
    },
    agent_protocol_module_loader_runtime::{
        emit_module_loader_runtime, emit_module_loader_runtime_selftest,
    },
    agent_protocol_module_reference::{
        emit_module_artifact_diagnostic, emit_module_artifact_diagnostic_selftest,
        emit_module_manifest_diagnostic, emit_module_manifest_diagnostic_selftest,
        emit_module_vm_report_diagnostic, emit_module_vm_report_diagnostic_selftest,
    },
    agent_protocol_module_service_slot::{
        emit_module_service_slot_diagnostic, emit_module_service_slot_diagnostic_selftest,
    },
    agent_protocol_module_service_slot_allocator::{
        emit_module_service_slot_allocator, emit_module_service_slot_allocator_selftest,
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
    },
    agent_protocol_policy::{emit_capability_denied, record_denial, record_read},
    agent_protocol_provider::{
        emit_provider_context_export_authorized_selftest,
        emit_provider_context_export_authorized_selftest_smuggle,
        emit_provider_context_export_denied, emit_provider_context_export_packet_selftest,
        emit_provider_context_gate, emit_provider_context_gate_selftest,
        emit_provider_context_injection_gate, emit_provider_context_injection_gate_selftest,
    },
    agent_protocol_recovery::{
        emit_durable_audit_rollback_write_authority_diagnostic,
        emit_durable_audit_rollback_write_authority_diagnostic_selftest,
        emit_recovery_artifact_identity_diagnostic,
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
        emit_recovery_disable_module_target_binding_diagnostic,
        emit_recovery_disable_module_target_binding_diagnostic_selftest,
        emit_recovery_durable_audit_rollback_persistence,
        emit_recovery_durable_audit_rollback_persistence_selftest,
        emit_recovery_lifeline_command_admission,
        emit_recovery_lifeline_command_admission_selftest,
        emit_recovery_lifeline_command_body_canonicalization_diagnostic,
        emit_recovery_lifeline_command_body_canonicalization_diagnostic_selftest,
        emit_recovery_lifeline_command_dispatch_behavior_diagnostic,
        emit_recovery_lifeline_command_dispatch_behavior_diagnostic_selftest,
        emit_recovery_lifeline_command_dispatch_diagnostic,
        emit_recovery_lifeline_command_dispatch_diagnostic_selftest,
        emit_recovery_lifeline_command_envelope_diagnostic,
        emit_recovery_lifeline_command_envelope_diagnostic_selftest,
        emit_recovery_lifeline_command_executor_capability_table_diagnostic,
        emit_recovery_lifeline_command_executor_capability_table_diagnostic_selftest,
        emit_recovery_lifeline_command_handler_binding_diagnostic,
        emit_recovery_lifeline_command_handler_binding_diagnostic_selftest,
        emit_recovery_lifeline_command_side_effect_gate_diagnostic,
        emit_recovery_lifeline_command_side_effect_gate_diagnostic_selftest,
        emit_recovery_lifeline_command_vocabulary,
        emit_recovery_lifeline_command_vocabulary_selftest,
        emit_recovery_lifeline_protocol_diagnostic,
        emit_recovery_lifeline_protocol_diagnostic_selftest,
        emit_recovery_lifeline_request_diagnostic,
        emit_recovery_lifeline_request_diagnostic_selftest,
        emit_recovery_lifeline_status_read_handler_diagnostic,
        emit_recovery_lifeline_status_read_handler_diagnostic_selftest,
        emit_recovery_load_artifact_by_hash_target_binding_diagnostic,
        emit_recovery_load_artifact_by_hash_target_binding_diagnostic_selftest,
        emit_recovery_loader_runtime_isolation, emit_recovery_loader_runtime_isolation_selftest,
        emit_recovery_memory_provenance, emit_recovery_memory_provenance_selftest,
        emit_recovery_memory_write_authority_diagnostic,
        emit_recovery_memory_write_authority_diagnostic_selftest,
        emit_recovery_restart_last_good_target_binding_diagnostic,
        emit_recovery_restart_last_good_target_binding_diagnostic_selftest,
        emit_recovery_rollback_apply_authorization_diagnostic,
        emit_recovery_rollback_apply_authorization_diagnostic_selftest,
        emit_recovery_rollback_preview_authorization_diagnostic,
        emit_recovery_rollback_preview_authorization_diagnostic_selftest,
        emit_recovery_rollback_transaction_engine,
        emit_recovery_rollback_transaction_engine_selftest,
        emit_recovery_service_inventory_side_effect_boundary_diagnostic,
        emit_recovery_service_inventory_side_effect_boundary_diagnostic_selftest,
    },
    agent_protocol_recovery_execution::{
        emit_recovery_lifeline_command_execution_audit_denial_diagnostic,
        emit_recovery_lifeline_command_execution_audit_denial_diagnostic_selftest,
        emit_recovery_lifeline_command_execution_commit_gate_diagnostic,
        emit_recovery_lifeline_command_execution_commit_gate_diagnostic_selftest,
        emit_recovery_lifeline_command_execution_completion_denial_diagnostic,
        emit_recovery_lifeline_command_execution_completion_denial_diagnostic_selftest,
        emit_recovery_lifeline_command_execution_enablement_diagnostic,
        emit_recovery_lifeline_command_execution_enablement_diagnostic_selftest,
        emit_recovery_lifeline_command_execution_intent_diagnostic,
        emit_recovery_lifeline_command_execution_intent_diagnostic_selftest,
        emit_recovery_lifeline_command_execution_observation_denial_diagnostic,
        emit_recovery_lifeline_command_execution_observation_denial_diagnostic_selftest,
        emit_recovery_lifeline_command_execution_preflight_diagnostic,
        emit_recovery_lifeline_command_execution_preflight_diagnostic_selftest,
        emit_recovery_lifeline_command_execution_result_denial_diagnostic,
        emit_recovery_lifeline_command_execution_result_denial_diagnostic_selftest,
        emit_recovery_lifeline_status_execution_result_diagnostic,
        emit_recovery_lifeline_status_result_read,
    },
    agent_protocol_support::{method_eq, method_head_eq},
    agent_protocol_system::{
        emit_boot_log, emit_capabilities, emit_describe, emit_device_graph, emit_persist_layout,
        emit_problem_list, emit_service_inventory, emit_snapshot,
    },
    agent_protocol_wasm::{
        emit_submit_candidate_chunk, emit_submit_candidate_finalize, emit_wasm_echo_probe,
    },
    echo_service, event_log, granted_candidate_service, hello_service, memory_store, ui,
};

pub(crate) use crate::agent_protocol_provider::provider_minimal_context_evidence_for_runtime;

pub enum DispatchOutcome {
    Response(&'static str),
    Denied(&'static str),
    Unknown,
}

type MethodHandler = fn(MethodCall<'_>, ui::RuntimeStatus) -> DispatchOutcome;
type MethodPredicate = fn(&str) -> bool;

#[derive(Clone, Copy)]
pub(crate) struct CommandEnvelopeTarget {
    pub(crate) method: &'static str,
    pub(crate) capability: &'static str,
    pub(crate) response_id: &'static str,
    pub(crate) dispatch_method: &'static str,
}

#[derive(Clone, Copy)]
struct CommandEnvelopeMetadata {
    order: u8,
    target: CommandEnvelopeTarget,
}

#[derive(Clone, Copy)]
struct ConsoleRoute {
    command: &'static str,
    dispatch_method: Option<&'static str>,
}

#[derive(Clone, Copy)]
enum MatchKind {
    Exact,
    Head,
    Predicate(MethodPredicate),
}

#[derive(Clone, Copy)]
enum MethodAction {
    Read0(fn()),
    ReadRuntime(fn(ui::RuntimeStatus)),
    ReadMethod(fn(&str)),
    ReadRuntimeMethod(fn(ui::RuntimeStatus, &str)),
    ReadRuntimeMethodEvent(fn(ui::RuntimeStatus, &str, event_log::EventId)),
    Response0Read(fn() -> &'static str),
    ResponseMethod(fn(&str) -> &'static str),
    ResponseMethodReadEvent(fn(&str, event_log::EventId) -> &'static str),
    ResponseMaterializeDryRun(fn(&str, event_log::EventId) -> &'static str),
    DeniedMethod(fn(&str) -> &'static str),
    DeniedProviderContextExport,
    DeniedModuleLoadEphemeral,
    DeniedRecoveryArtifactLoad,
    DeniedMemoryMutation,
    DeniedGeneric,
    StatusResultRead,
}

struct MethodEntry {
    canonical: &'static str,
    aliases: &'static [&'static str],
    match_kind: MatchKind,
    envelope: Option<CommandEnvelopeMetadata>,
    console_routes: &'static [ConsoleRoute],
    action: MethodAction,
    handler: MethodHandler,
}

struct MethodCall<'a> {
    input: &'a str,
    canonical: &'static str,
    entry: &'static MethodEntry,
}

macro_rules! route {
    ($command:literal => $dispatch:literal) => {
        ConsoleRoute {
            command: $command,
            dispatch_method: Some($dispatch),
        }
    };
    ($command:literal) => {
        ConsoleRoute {
            command: $command,
            dispatch_method: None,
        }
    };
}

macro_rules! method {
    ($canonical:literal, $kind:ident, [$($alias:literal),* $(,)?], [$($route:expr),* $(,)?], $action:expr) => {
        MethodEntry {
            canonical: $canonical,
            aliases: &[$($alias),*],
            match_kind: MatchKind::$kind,
            envelope: None,
            console_routes: &[$($route),*],
            action: $action,
            handler: dispatch_method_entry,
        }
    };
}

macro_rules! envelope_method {
    ($canonical:literal, $kind:ident, [$($alias:literal),* $(,)?], [$($route:expr),* $(,)?], $order:literal, $target:literal, $capability:literal, $response:literal, $dispatch:literal, $action:expr) => {
        MethodEntry {
            canonical: $canonical,
            aliases: &[$($alias),*],
            match_kind: MatchKind::$kind,
            envelope: Some(CommandEnvelopeMetadata {
                order: $order,
                target: CommandEnvelopeTarget {
                    method: $target,
                    capability: $capability,
                    response_id: $response,
                    dispatch_method: $dispatch,
                },
            }),
            console_routes: &[$($route),*],
            action: $action,
            handler: dispatch_method_entry,
        }
    };
}

macro_rules! pred_method {
    ($canonical:literal, $predicate:expr, [$($route:expr),* $(,)?], $action:expr) => {
        MethodEntry {
            canonical: $canonical,
            aliases: &[],
            match_kind: MatchKind::Predicate($predicate),
            envelope: None,
            console_routes: &[$($route),*],
            action: $action,
            handler: dispatch_method_entry,
        }
    };
}

macro_rules! pred_envelope_method {
    ($canonical:literal, $predicate:expr, [$($route:expr),* $(,)?], $order:literal, $target:literal, $capability:literal, $response:literal, $dispatch:literal, $action:expr) => {
        MethodEntry {
            canonical: $canonical,
            aliases: &[],
            match_kind: MatchKind::Predicate($predicate),
            envelope: Some(CommandEnvelopeMetadata {
                order: $order,
                target: CommandEnvelopeTarget {
                    method: $target,
                    capability: $capability,
                    response_id: $response,
                    dispatch_method: $dispatch,
                },
            }),
            console_routes: &[$($route),*],
            action: $action,
            handler: dispatch_method_entry,
        }
    };
}

#[rustfmt::skip]
const AGENT_METHODS: &[MethodEntry] = &[
    envelope_method!("system.describe", Exact, ["describe"], [route!("describe" => "system.describe"), route!("system.describe" => "system.describe")], 0, "system.describe", "cap.system.describe.read", "agent_command_envelope.current_boot.serial.system_describe.v0", "system.describe", MethodAction::Read0(emit_describe)),
    envelope_method!("system.snapshot", Exact, ["snapshot"], [route!("snapshot" => "system.snapshot"), route!("system.snapshot" => "system.snapshot")], 1, "system.snapshot", "cap.system.snapshot.read", "agent_command_envelope.current_boot.serial.system_snapshot.v0", "system.snapshot", MethodAction::ReadRuntime(emit_snapshot)),
    envelope_method!("system.boot_log", Exact, ["system.bootlog", "bootlog"], [route!("bootlog" => "system.boot_log"), route!("system.bootlog" => "system.boot_log"), route!("system.boot_log" => "system.boot_log")], 2, "system.boot_log", "cap.system.boot_log.read", "agent_command_envelope.current_boot.serial.system_boot_log.v0", "system.boot_log", MethodAction::Read0(emit_boot_log)),
    envelope_method!("system.capabilities", Exact, ["capabilities", "caps"], [route!("caps" => "system.capabilities"), route!("capabilities" => "system.capabilities"), route!("system.capabilities" => "system.capabilities")], 3, "system.capabilities", "cap.system.capabilities.read", "agent_command_envelope.current_boot.serial.system_capabilities.v0", "system.capabilities", MethodAction::Read0(emit_capabilities)),
    envelope_method!("device.graph", Exact, ["devicegraph"], [route!("devicegraph" => "device.graph"), route!("device.graph" => "device.graph")], 4, "device.graph", "cap.device.graph.read", "agent_command_envelope.current_boot.serial.device_graph.v0", "device.graph", MethodAction::ReadRuntime(emit_device_graph)),
    method!("persist.layout", Exact, ["system.persist_layout"], [route!("persist.layout"), route!("system.persist_layout" => "persist.layout")], MethodAction::Read0(emit_persist_layout)),
    method!("durable.record_log_scan", Exact, ["persist.reclog_scan"], [route!("durable.record_log_scan"), route!("persist.reclog_scan" => "durable.record_log_scan")], MethodAction::Read0(durable_store::emit_durable_record_log_scan)),
    method!("durable.record_log_append", Exact, ["persist.reclog_append"], [route!("durable.record_log_append"), route!("persist.reclog_append" => "durable.record_log_append")], MethodAction::Read0(durable_store::emit_durable_record_log_append)),
    method!("module.promotion_transaction_selftest", Exact, [], [route!("module.promotion_transaction_selftest")], MethodAction::Read0(durable_store::emit_promotion_transaction_selftest)),
    method!("memory.record_log_append", Exact, ["persist.memory_record_append"], [route!("memory.record_log_append"), route!("persist.memory_record_append" => "memory.record_log_append")], MethodAction::Read0(memory_store::emit_memory_record_log_append)),
    method!("memory.record_log_append_selftest", Exact, [], [route!("memory.record_log_append_selftest")], MethodAction::Read0(memory_store::emit_memory_record_log_append_selftest)),
    method!("memory.broker_resolve_selftest", Exact, [], [route!("memory.broker_resolve_selftest")], MethodAction::Read0(memory_store::emit_memory_broker_resolve_selftest)),
    method!("memory.decision_problem_log_append", Exact, ["persist.memory_decision_problem_append"], [route!("memory.decision_problem_log_append"), route!("persist.memory_decision_problem_append" => "memory.decision_problem_log_append")], MethodAction::Read0(memory_store::emit_memory_decision_problem_log_append)),
    method!("memory.provider_export_public_fixture_append", Exact, [], [route!("memory.provider_export_public_fixture_append")], MethodAction::Read0(memory_store::emit_provider_export_public_fixture_append)),
    method!("memory.observation_log_append", Head, [], [], MethodAction::ReadMethod(memory_store::emit_memory_observation_log_append)),
    method!("artifact.store_scan", Exact, ["persist.artifact_store_scan"], [route!("artifact.store_scan"), route!("persist.artifact_store_scan" => "artifact.store_scan")], MethodAction::Read0(artifact_store::emit_artifact_store_scan)),
    method!("module.artifact_store_selftest", Exact, [], [route!("module.artifact_store_selftest")], MethodAction::Read0(artifact_store::emit_artifact_store_selftest)),
    method!("boot.control_read", Exact, ["persist.boot_control"], [route!("boot.control_read"), route!("persist.boot_control" => "boot.control_read")], MethodAction::Read0(boot_control::emit_boot_control_read)),
    method!("boot.control_mark_success", Exact, ["persist.boot_control_mark_success"], [route!("boot.control_mark_success"), route!("persist.boot_control_mark_success" => "boot.control_mark_success")], MethodAction::ReadRuntime(boot_control::emit_boot_control_success_mark)),
    method!("repromotion.run", Exact, ["persist.repromotion"], [route!("repromotion.run"), route!("persist.repromotion" => "repromotion.run")], MethodAction::ReadRuntime(repromotion::emit_repromotion_run)),
    envelope_method!("problem.list", Exact, ["problems"], [route!("problems" => "problem.list"), route!("problem.list" => "problem.list")], 17, "problem.list", "cap.problem.list.read", "agent_command_envelope.current_boot.serial.problem_list.v0", "problem.list", MethodAction::ReadRuntime(emit_problem_list)),
    envelope_method!("service.inventory", Exact, ["services"], [route!("services" => "service.inventory"), route!("service.inventory" => "service.inventory")], 5, "service.inventory", "cap.service.inventory.read", "agent_command_envelope.current_boot.serial.service_inventory.v0", "service.inventory", MethodAction::ReadRuntime(emit_service_inventory)),
    pred_method!("service.descriptor_source_trust_selftest", hello_service::is_descriptor_source_trust_selftest_method, [route!("service.descriptor_source_trust_selftest")], MethodAction::Response0Read(hello_service::emit_descriptor_source_trust_selftest)),
    pred_method!("service.artifact_reference_trust_selftest", hello_service::is_artifact_reference_trust_selftest_method, [route!("service.artifact_reference_trust_selftest")], MethodAction::Response0Read(hello_service::emit_artifact_reference_trust_selftest)),
    pred_method!("service.artifact_load_plan_preflight_selftest", hello_service::is_artifact_load_plan_preflight_selftest_method, [route!("service.artifact_load_plan_preflight_selftest")], MethodAction::Response0Read(hello_service::emit_artifact_load_plan_preflight_selftest)),
    pred_method!("module.granted_candidate_selftest", granted_candidate_service::is_selftest_method, [route!("module.granted_candidate_selftest")], MethodAction::Response0Read(granted_candidate_service::emit_selftest)),
    pred_envelope_method!("service.health", hello_service::is_health_method, [route!("service.health")], 6, "service.health", "cap.service.health.read", "agent_command_envelope.current_boot.serial.service_health.v0", "service.health svc.demo.hello", MethodAction::ResponseMethod(hello_service::emit_health)),
    pred_method!("service.health", echo_service::is_health_method, [], MethodAction::ResponseMethod(echo_service::emit_health)),
    method!("memory.profile", Exact, ["memprofile"], [route!("memory.profile" => "memory.profile"), route!("memprofile" => "memory.profile")], MethodAction::Read0(emit_memory_profile)),
    method!("memory.context", Head, ["memctx"], [route!("memory.context"), route!("memctx")], MethodAction::ReadRuntimeMethodEvent(emit_memory_context)),
    method!("memory.query", Head, ["memquery"], [route!("memory.query"), route!("memquery")], MethodAction::Read0(emit_memory_query)),
    method!("memory.trace", Head, ["memtrace"], [route!("memory.trace"), route!("memtrace")], MethodAction::ReadMethod(emit_memory_trace)),
    method!("memory.recent_events", Head, ["audit.events", "events"], [route!("memory.recent_events"), route!("audit.events"), route!("events")], MethodAction::ReadMethod(emit_recent_events)),
    method!("provider.context_gate", Head, ["provider.context_export_status"], [route!("provider.context_gate"), route!("provider.context_export_status")], MethodAction::ReadRuntimeMethod(emit_provider_context_gate)),
    method!("provider.context_gate_selftest", Head, [], [route!("provider.context_gate_selftest")], MethodAction::ReadRuntimeMethod(emit_provider_context_gate_selftest)),
    method!("provider.context_injection_gate", Head, [], [route!("provider.context_injection_gate")], MethodAction::ReadRuntimeMethod(emit_provider_context_injection_gate)),
    method!("provider.context_injection_gate_selftest", Head, [], [route!("provider.context_injection_gate_selftest")], MethodAction::ReadRuntimeMethod(emit_provider_context_injection_gate_selftest)),
    method!("provider.context_export_packet_selftest", Head, [], [route!("provider.context_export_packet_selftest")], MethodAction::ReadMethod(emit_provider_context_export_packet_selftest)),
    method!("provider.context_export_authorized_selftest", Head, [], [route!("provider.context_export_authorized_selftest")], MethodAction::ReadMethod(emit_provider_context_export_authorized_selftest)),
    method!("provider.context_export_authorized_selftest_smuggle", Head, [], [route!("provider.context_export_authorized_selftest_smuggle")], MethodAction::ReadMethod(emit_provider_context_export_authorized_selftest_smuggle)),
    method!("module.manifest_diagnostic", Head, [], [], MethodAction::ReadMethod(emit_module_manifest_diagnostic)),
    method!("module.manifest_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_module_manifest_diagnostic_selftest)),
    method!("module.artifact_diagnostic", Head, [], [], MethodAction::ReadMethod(emit_module_artifact_diagnostic)),
    method!("module.artifact_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_module_artifact_diagnostic_selftest)),
    method!("module.vm_report_diagnostic", Head, [], [], MethodAction::ReadMethod(emit_module_vm_report_diagnostic)),
    method!("module.vm_report_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_module_vm_report_diagnostic_selftest)),
    method!("module.attestation_diagnostic", Head, [], [], MethodAction::ReadMethod(emit_module_attestation_diagnostic)),
    method!("module.attestation_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_module_attestation_diagnostic_selftest)),
    method!("module.approval_diagnostic", Head, [], [], MethodAction::ReadMethod(emit_module_approval_diagnostic)),
    method!("module.approval_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_module_approval_diagnostic_selftest)),
    method!("module.grant_diagnostic", Head, [], [], MethodAction::ReadMethod(emit_module_grant_diagnostic)),
    method!("module.grant_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_module_grant_diagnostic_selftest)),
    method!("module.audit_rollback_diagnostic", Head, [], [], MethodAction::ReadMethod(emit_module_audit_rollback_diagnostic)),
    method!("module.audit_rollback_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_module_audit_rollback_diagnostic_selftest)),
    method!("module.service_slot_diagnostic", Head, [], [], MethodAction::ReadMethod(emit_module_service_slot_diagnostic)),
    method!("module.service_slot_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_module_service_slot_diagnostic_selftest)),
    method!("module.service_slot_allocator", Head, [], [], MethodAction::Read0(emit_module_service_slot_allocator)),
    method!("module.service_slot_allocator_selftest", Head, [], [], MethodAction::Read0(emit_module_service_slot_allocator_selftest)),
    method!("module.loader_runtime", Head, [], [], MethodAction::Read0(emit_module_loader_runtime)),
    method!("module.loader_runtime_selftest", Head, [], [], MethodAction::Read0(emit_module_loader_runtime_selftest)),
    method!("module.loader_identity", Head, [], [], MethodAction::Read0(emit_module_loader_identity)),
    method!("module.loader_identity_selftest", Head, [], [], MethodAction::Read0(emit_module_loader_identity_selftest)),
    method!("module.loader_artifact_hash_binding", Head, [], [], MethodAction::Read0(emit_module_loader_artifact_hash_binding)),
    method!("module.loader_artifact_hash_binding_selftest", Head, [], [], MethodAction::Read0(emit_module_loader_artifact_hash_binding_selftest)),
    method!("module.loader_entrypoint_abi", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact)),
    method!("module.loader_entrypoint_abi_selftest", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact_selftest)),
    method!("module.loader_address_space_boundary", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact)),
    method!("module.loader_address_space_boundary_selftest", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact_selftest)),
    method!("module.loader_memory_map_constraints", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact)),
    method!("module.loader_memory_map_constraints_selftest", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact_selftest)),
    method!("module.loader_capability_import_table", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact)),
    method!("module.loader_capability_import_table_selftest", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact_selftest)),
    method!("module.loader_service_slot_binding", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact)),
    method!("module.loader_service_slot_binding_selftest", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact_selftest)),
    method!("module.loader_health_state_hooks", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact)),
    method!("module.loader_health_state_hooks_selftest", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact_selftest)),
    method!("module.loader_rollback_hooks", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact)),
    method!("module.loader_rollback_hooks_selftest", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact_selftest)),
    method!("module.loader_audit_rollback_write_boundary_binding", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact)),
    method!("module.loader_audit_rollback_write_boundary_binding_selftest", Head, [], [], MethodAction::ReadMethod(emit_module_loader_fact_selftest)),
    method!("wasm.echo_probe", Exact, [], [route!("wasm.echo_probe")], MethodAction::Read0(emit_wasm_echo_probe)),
    method!("echo.invoke_fuel_starved", Exact, [], [route!("echo.invoke_fuel_starved")], MethodAction::Read0(echo_service::emit_invoke_fuel_starved)),
    method!("module.submit_candidate_chunk", Head, [], [route!("module.submit_candidate_chunk")], MethodAction::ReadMethod(emit_submit_candidate_chunk)),
    method!("module.submit_candidate_finalize", Exact, [], [route!("module.submit_candidate_finalize")], MethodAction::Read0(emit_submit_candidate_finalize)),
    envelope_method!("module.audit_rollback_availability", Head, ["module.audit_rollback_store_availability"], [], 9, "module.audit_rollback_availability", "cap.module.grant_diagnostic.read", "agent_command_envelope.current_boot.serial.module_audit_rollback_availability.v0", "module.audit_rollback_availability", MethodAction::Read0(emit_module_audit_rollback_availability)),
    method!("module.audit_rollback_availability_selftest", Head, ["module.audit_rollback_store_availability_selftest"], [], MethodAction::Read0(emit_module_audit_rollback_availability_selftest)),
    envelope_method!("module.audit_rollback_write_policy", Head, ["module.audit_rollback_policy"], [], 10, "module.audit_rollback_write_policy", "cap.module.grant_diagnostic.read", "agent_command_envelope.current_boot.serial.module_audit_rollback_write_policy.v0", "module.audit_rollback_write_policy", MethodAction::Read0(emit_module_audit_rollback_write_policy)),
    method!("module.audit_rollback_write_policy_selftest", Head, ["module.audit_rollback_policy_selftest"], [], MethodAction::Read0(emit_module_audit_rollback_write_policy_selftest)),
    envelope_method!("module.audit_rollback_storage_layout", Head, ["module.audit_rollback_persistence_layout"], [], 11, "module.audit_rollback_storage_layout", "cap.module.grant_diagnostic.read", "agent_command_envelope.current_boot.serial.module_audit_rollback_storage_layout.v0", "module.audit_rollback_storage_layout", MethodAction::Read0(emit_module_audit_rollback_storage_layout)),
    method!("module.audit_rollback_storage_layout_selftest", Head, ["module.audit_rollback_persistence_layout_selftest"], [], MethodAction::Read0(emit_module_audit_rollback_storage_layout_selftest)),
    envelope_method!("module.audit_rollback_append_engine", Head, ["module.audit_rollback_append_engine_readiness"], [], 12, "module.audit_rollback_append_engine", "cap.module.grant_diagnostic.read", "agent_command_envelope.current_boot.serial.module_audit_rollback_append_engine.v0", "module.audit_rollback_append_engine", MethodAction::Read0(emit_module_audit_rollback_append_engine)),
    method!("module.audit_rollback_append_engine_selftest", Head, ["module.audit_rollback_append_engine_readiness_selftest"], [], MethodAction::Read0(emit_module_audit_rollback_append_engine_selftest)),
    envelope_method!("module.audit_rollback_append_contract", Head, ["module.audit_rollback_storage_contract"], [], 13, "module.audit_rollback_append_contract", "cap.module.grant_diagnostic.read", "agent_command_envelope.current_boot.serial.module_audit_rollback_append_contract.v0", "module.audit_rollback_append_contract", MethodAction::Read0(emit_module_audit_rollback_append_contract)),
    method!("module.audit_rollback_append_contract_selftest", Head, ["module.audit_rollback_storage_contract_selftest"], [], MethodAction::Read0(emit_module_audit_rollback_append_contract_selftest)),
    envelope_method!("module.audit_rollback_append_payload_hash", Head, ["module.audit_rollback_append_payload"], [], 14, "module.audit_rollback_append_payload_hash", "cap.module.grant_diagnostic.read", "agent_command_envelope.current_boot.serial.module_audit_rollback_append_payload_hash.v0", "module.audit_rollback_append_payload_hash", MethodAction::Read0(emit_module_audit_rollback_append_payload_hash)),
    method!("module.audit_rollback_append_payload_hash_selftest", Head, ["module.audit_rollback_append_payload_selftest"], [], MethodAction::Read0(emit_module_audit_rollback_append_payload_hash_selftest)),
    envelope_method!("module.audit_rollback_append_intent", Head, ["module.audit_rollback_append_request"], [], 15, "module.audit_rollback_append_intent", "cap.module.grant_diagnostic.read", "agent_command_envelope.current_boot.serial.module_audit_rollback_append_intent.v0", "module.audit_rollback_append_intent", MethodAction::Read0(emit_module_audit_rollback_append_intent)),
    method!("module.audit_rollback_append_intent_selftest", Head, ["module.audit_rollback_append_request_selftest"], [], MethodAction::Read0(emit_module_audit_rollback_append_intent_selftest)),
    envelope_method!("module.audit_rollback_write_boundary", Head, ["module.audit_rollback_write_gate"], [], 16, "module.audit_rollback_write_boundary", "cap.module.grant_diagnostic.read", "agent_command_envelope.current_boot.serial.module_audit_rollback_write_boundary.v0", "module.audit_rollback_write_boundary", MethodAction::Read0(emit_module_audit_rollback_write_boundary)),
    method!("module.audit_rollback_write_boundary_selftest", Head, ["module.audit_rollback_write_gate_selftest"], [], MethodAction::Read0(emit_module_audit_rollback_write_boundary_selftest)),
    method!("module.load_gate_manifest_selftest", Head, [], [], MethodAction::Read0(emit_module_load_gate_manifest_selftest)),
    method!("module.load_gate_artifact_selftest", Head, [], [], MethodAction::Read0(emit_module_load_gate_artifact_selftest)),
    method!("module.load_gate_vm_report_selftest", Head, [], [], MethodAction::Read0(emit_module_load_gate_vm_report_selftest)),
    method!("module.load_gate_attestation_selftest", Head, [], [], MethodAction::Read0(emit_module_load_gate_attestation_selftest)),
    method!("module.load_gate_approval_selftest", Head, [], [], MethodAction::Read0(emit_module_load_gate_approval_selftest)),
    method!("module.load_gate_retained_selftest", Head, [], [], MethodAction::Read0(emit_module_load_gate_retained_selftest)),
    method!("module.load_gate_audit_rollback_selftest", Head, [], [], MethodAction::Read0(emit_module_load_gate_audit_rollback_selftest)),
    method!("module.load_gate_service_slot_selftest", Head, [], [], MethodAction::Read0(emit_module_load_gate_service_slot_selftest)),
    method!("module.load_gate_loader_runtime_selftest", Head, [], [], MethodAction::Read0(emit_module_load_gate_loader_runtime_selftest)),
    method!("recovery.identity_diagnostic", Head, [], [], MethodAction::ReadMethod(emit_recovery_artifact_identity_diagnostic)),
    method!("recovery.identity_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_recovery_artifact_identity_diagnostic_selftest)),
    method!("recovery.trust_diagnostic", Head, [], [], MethodAction::ReadMethod(emit_recovery_artifact_trust_diagnostic)),
    method!("recovery.trust_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_recovery_artifact_trust_diagnostic_selftest)),
    method!("recovery.vm_test_diagnostic", Head, [], [], MethodAction::ReadMethod(emit_recovery_artifact_vm_test_diagnostic)),
    method!("recovery.vm_test_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_recovery_artifact_vm_test_diagnostic_selftest)),
    method!("recovery.local_approval_diagnostic", Head, [], [], MethodAction::ReadMethod(emit_recovery_artifact_local_approval_diagnostic)),
    method!("recovery.local_approval_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_recovery_artifact_local_approval_diagnostic_selftest)),
    method!("recovery.loader_diagnostic", Head, [], [], MethodAction::ReadMethod(emit_recovery_artifact_loader_diagnostic)),
    method!("recovery.loader_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_recovery_artifact_loader_diagnostic_selftest)),
    method!("recovery.rollback_evidence_diagnostic", Head, [], [], MethodAction::ReadMethod(emit_recovery_artifact_rollback_evidence_diagnostic)),
    method!("recovery.rollback_evidence_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_recovery_artifact_rollback_evidence_diagnostic_selftest)),
    pred_envelope_method!("recovery.rollback_inspect", hello_service::is_recovery_rollback_inspect_method, [route!("recovery.rollback_inspect")], 8, "recovery.rollback_inspect", "cap.recovery.rollback_inspect.read", "agent_command_envelope.current_boot.serial.recovery_rollback_inspect.v0", "recovery.rollback_inspect svc.demo.hello", MethodAction::ResponseMethodReadEvent(hello_service::emit_recovery_rollback_inspect)),
    method!("recovery.lifeline_request_diagnostic", Head, [], [], MethodAction::ReadMethod(emit_recovery_lifeline_request_diagnostic)),
    method!("recovery.lifeline_request_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_recovery_lifeline_request_diagnostic_selftest)),
    method!("recovery.lifeline_protocol_diagnostic", Head, [], [], MethodAction::Read0(emit_recovery_lifeline_protocol_diagnostic)),
    method!("recovery.lifeline_protocol_diagnostic_selftest", Head, [], [], MethodAction::Read0(emit_recovery_lifeline_protocol_diagnostic_selftest)),
    method!("recovery.lifeline_command_vocabulary", Head, ["recovery.lifeline_command_vocabulary_diagnostic"], [], MethodAction::Read0(emit_recovery_lifeline_command_vocabulary)),
    method!("recovery.lifeline_command_vocabulary_selftest", Head, ["recovery.lifeline_command_vocabulary_diagnostic_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_vocabulary_selftest)),
    method!("recovery.loader_runtime_isolation", Head, ["recovery.loader_runtime_isolation_diagnostic"], [], MethodAction::Read0(emit_recovery_loader_runtime_isolation)),
    method!("recovery.loader_runtime_isolation_selftest", Head, ["recovery.loader_runtime_isolation_diagnostic_selftest"], [], MethodAction::Read0(emit_recovery_loader_runtime_isolation_selftest)),
    method!("recovery.rollback_transaction_engine", Head, ["recovery.rollback_transaction_engine_diagnostic"], [], MethodAction::Read0(emit_recovery_rollback_transaction_engine)),
    method!("recovery.rollback_transaction_engine_selftest", Head, ["recovery.rollback_transaction_engine_diagnostic_selftest"], [], MethodAction::Read0(emit_recovery_rollback_transaction_engine_selftest)),
    method!("recovery.durable_audit_rollback_persistence", Head, ["recovery.durable_audit_rollback_persistence_diagnostic"], [], MethodAction::Read0(emit_recovery_durable_audit_rollback_persistence)),
    method!("recovery.durable_audit_rollback_persistence_selftest", Head, ["recovery.durable_audit_rollback_persistence_diagnostic_selftest"], [], MethodAction::Read0(emit_recovery_durable_audit_rollback_persistence_selftest)),
    method!("recovery.memory_provenance", Head, ["recovery.memory_provenance_diagnostic"], [], MethodAction::Read0(emit_recovery_memory_provenance)),
    method!("recovery.memory_provenance_selftest", Head, ["recovery.memory_provenance_diagnostic_selftest"], [], MethodAction::Read0(emit_recovery_memory_provenance_selftest)),
    method!("recovery.lifeline_command_admission", Head, ["recovery.lifeline_command_admission_diagnostic"], [], MethodAction::Read0(emit_recovery_lifeline_command_admission)),
    method!("recovery.lifeline_command_admission_selftest", Head, ["recovery.lifeline_command_admission_diagnostic_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_admission_selftest)),
    method!("recovery.lifeline_command_envelope_diagnostic", Head, ["recovery.lifeline_command_envelope_reference"], [], MethodAction::ReadMethod(emit_recovery_lifeline_command_envelope_diagnostic)),
    method!("recovery.lifeline_command_envelope_diagnostic_selftest", Head, ["recovery.lifeline_command_envelope_reference_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_envelope_diagnostic_selftest)),
    method!("recovery.lifeline_command_dispatch_diagnostic", Head, ["recovery.lifeline_command_dispatch_denial"], [], MethodAction::Read0(emit_recovery_lifeline_command_dispatch_diagnostic)),
    method!("recovery.lifeline_command_dispatch_diagnostic_selftest", Head, ["recovery.lifeline_command_dispatch_denial_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_dispatch_diagnostic_selftest)),
    method!("recovery.lifeline_command_body_canonicalization_diagnostic", Head, ["recovery.lifeline_command_body_canonicalization"], [], MethodAction::ReadMethod(emit_recovery_lifeline_command_body_canonicalization_diagnostic)),
    method!("recovery.lifeline_command_body_canonicalization_diagnostic_selftest", Head, ["recovery.lifeline_command_body_canonicalization_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_body_canonicalization_diagnostic_selftest)),
    method!("recovery.lifeline_command_handler_binding_diagnostic", Head, ["recovery.lifeline_command_handler_binding"], [], MethodAction::ReadMethod(emit_recovery_lifeline_command_handler_binding_diagnostic)),
    method!("recovery.lifeline_command_handler_binding_diagnostic_selftest", Head, ["recovery.lifeline_command_handler_binding_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_handler_binding_diagnostic_selftest)),
    method!("recovery.lifeline_status_read_handler_diagnostic", Head, ["recovery.lifeline_status_read_handler"], [], MethodAction::ReadMethod(emit_recovery_lifeline_status_read_handler_diagnostic)),
    method!("recovery.lifeline_status_read_handler_diagnostic_selftest", Head, ["recovery.lifeline_status_read_handler_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_status_read_handler_diagnostic_selftest)),
    method!("recovery.rollback_preview_authorization_diagnostic", Head, ["recovery.rollback_preview_authorization"], [], MethodAction::ReadMethod(emit_recovery_rollback_preview_authorization_diagnostic)),
    method!("recovery.rollback_preview_authorization_diagnostic_selftest", Head, ["recovery.rollback_preview_authorization_selftest"], [], MethodAction::Read0(emit_recovery_rollback_preview_authorization_diagnostic_selftest)),
    method!("recovery.rollback_apply_authorization_diagnostic", Head, ["recovery.rollback_apply_authorization"], [], MethodAction::ReadMethod(emit_recovery_rollback_apply_authorization_diagnostic)),
    method!("recovery.rollback_apply_authorization_diagnostic_selftest", Head, ["recovery.rollback_apply_authorization_selftest"], [], MethodAction::Read0(emit_recovery_rollback_apply_authorization_diagnostic_selftest)),
    method!("recovery.disable_module_target_binding_diagnostic", Head, ["recovery.disable_module_target_binding"], [], MethodAction::ReadMethod(emit_recovery_disable_module_target_binding_diagnostic)),
    method!("recovery.disable_module_target_binding_diagnostic_selftest", Head, ["recovery.disable_module_target_binding_selftest"], [], MethodAction::Read0(emit_recovery_disable_module_target_binding_diagnostic_selftest)),
    method!("recovery.disable_module_selftest", Exact, [], [route!("recovery.disable_module_selftest")], MethodAction::Read0(durable_store::emit_recovery_action_selftest)),
    method!("recovery.restart_last_good_target_binding_diagnostic", Head, ["recovery.restart_last_good_target_binding"], [], MethodAction::ReadMethod(emit_recovery_restart_last_good_target_binding_diagnostic)),
    method!("recovery.restart_last_good_target_binding_diagnostic_selftest", Head, ["recovery.restart_last_good_target_binding_selftest"], [], MethodAction::Read0(emit_recovery_restart_last_good_target_binding_diagnostic_selftest)),
    method!("recovery.load_artifact_by_hash_target_binding_diagnostic", Head, ["recovery.load_artifact_by_hash_target_binding"], [], MethodAction::ReadMethod(emit_recovery_load_artifact_by_hash_target_binding_diagnostic)),
    method!("recovery.load_artifact_by_hash_target_binding_diagnostic_selftest", Head, ["recovery.load_artifact_by_hash_target_binding_selftest"], [], MethodAction::Read0(emit_recovery_load_artifact_by_hash_target_binding_diagnostic_selftest)),
    method!("recovery.load_artifact_by_hash_selftest", Exact, [], [route!("recovery.load_artifact_by_hash_selftest")], MethodAction::Read0(recovery_lifeline::emit_load_artifact_by_hash_selftest)),
    method!("recovery.memory_write_authority_diagnostic", Head, ["recovery.memory_write_authority"], [], MethodAction::ReadMethod(emit_recovery_memory_write_authority_diagnostic)),
    method!("recovery.memory_write_authority_diagnostic_selftest", Head, ["recovery.memory_write_authority_selftest"], [], MethodAction::Read0(emit_recovery_memory_write_authority_diagnostic_selftest)),
    method!("recovery.durable_audit_rollback_write_authority_diagnostic", Head, ["recovery.durable_audit_rollback_write_authority"], [], MethodAction::ReadMethod(emit_durable_audit_rollback_write_authority_diagnostic)),
    method!("recovery.durable_audit_rollback_write_authority_diagnostic_selftest", Head, ["recovery.durable_audit_rollback_write_authority_selftest"], [], MethodAction::Read0(emit_durable_audit_rollback_write_authority_diagnostic_selftest)),
    method!("recovery.service_inventory_side_effect_boundary_diagnostic", Head, ["recovery.service_inventory_side_effect_boundary"], [], MethodAction::ReadMethod(emit_recovery_service_inventory_side_effect_boundary_diagnostic)),
    method!("recovery.service_inventory_side_effect_boundary_diagnostic_selftest", Head, ["recovery.service_inventory_side_effect_boundary_selftest"], [], MethodAction::Read0(emit_recovery_service_inventory_side_effect_boundary_diagnostic_selftest)),
    method!("recovery.lifeline_command_dispatch_behavior_diagnostic", Head, ["recovery.lifeline_command_dispatch_behavior"], [], MethodAction::ReadMethod(emit_recovery_lifeline_command_dispatch_behavior_diagnostic)),
    method!("recovery.lifeline_command_dispatch_behavior_diagnostic_selftest", Head, ["recovery.lifeline_command_dispatch_behavior_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_dispatch_behavior_diagnostic_selftest)),
    method!("recovery.lifeline_command_executor_capability_table_diagnostic", Head, ["recovery.lifeline_command_executor_capability_table"], [], MethodAction::ReadMethod(emit_recovery_lifeline_command_executor_capability_table_diagnostic)),
    method!("recovery.lifeline_command_executor_capability_table_diagnostic_selftest", Head, ["recovery.lifeline_command_executor_capability_table_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_executor_capability_table_diagnostic_selftest)),
    method!("recovery.lifeline_command_side_effect_gate_diagnostic", Head, ["recovery.lifeline_command_side_effect_gate"], [], MethodAction::ReadMethod(emit_recovery_lifeline_command_side_effect_gate_diagnostic)),
    method!("recovery.lifeline_command_side_effect_gate_diagnostic_selftest", Head, ["recovery.lifeline_command_side_effect_gate_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_side_effect_gate_diagnostic_selftest)),
    method!("recovery.lifeline_command_execution_enablement_diagnostic", Head, ["recovery.lifeline_command_execution_enablement"], [], MethodAction::ReadMethod(emit_recovery_lifeline_command_execution_enablement_diagnostic)),
    method!("recovery.lifeline_command_execution_enablement_diagnostic_selftest", Head, ["recovery.lifeline_command_execution_enablement_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_execution_enablement_diagnostic_selftest)),
    method!("recovery.lifeline_command_execution_preflight_diagnostic", Head, ["recovery.lifeline_command_execution_preflight"], [], MethodAction::ReadMethod(emit_recovery_lifeline_command_execution_preflight_diagnostic)),
    method!("recovery.lifeline_command_execution_preflight_diagnostic_selftest", Head, ["recovery.lifeline_command_execution_preflight_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_execution_preflight_diagnostic_selftest)),
    method!("recovery.lifeline_command_execution_intent_diagnostic", Head, ["recovery.lifeline_command_execution_intent"], [], MethodAction::ReadMethod(emit_recovery_lifeline_command_execution_intent_diagnostic)),
    method!("recovery.lifeline_command_execution_intent_diagnostic_selftest", Head, ["recovery.lifeline_command_execution_intent_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_execution_intent_diagnostic_selftest)),
    method!("recovery.lifeline_command_execution_commit_gate_diagnostic", Head, ["recovery.lifeline_command_execution_commit_gate"], [], MethodAction::ReadMethod(emit_recovery_lifeline_command_execution_commit_gate_diagnostic)),
    method!("recovery.lifeline_command_execution_commit_gate_diagnostic_selftest", Head, ["recovery.lifeline_command_execution_commit_gate_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_execution_commit_gate_diagnostic_selftest)),
    method!("recovery.lifeline_command_execution_result_denial_diagnostic", Head, ["recovery.lifeline_command_execution_result_denial"], [], MethodAction::ReadMethod(emit_recovery_lifeline_command_execution_result_denial_diagnostic)),
    method!("recovery.lifeline_command_execution_result_denial_diagnostic_selftest", Head, ["recovery.lifeline_command_execution_result_denial_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_execution_result_denial_diagnostic_selftest)),
    method!("recovery.lifeline_command_execution_audit_denial_diagnostic", Head, ["recovery.lifeline_command_execution_audit_denial"], [], MethodAction::ReadMethod(emit_recovery_lifeline_command_execution_audit_denial_diagnostic)),
    method!("recovery.lifeline_command_execution_audit_denial_diagnostic_selftest", Head, ["recovery.lifeline_command_execution_audit_denial_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_execution_audit_denial_diagnostic_selftest)),
    method!("recovery.lifeline_command_execution_observation_denial_diagnostic", Head, ["recovery.lifeline_command_execution_observation_denial"], [], MethodAction::ReadMethod(emit_recovery_lifeline_command_execution_observation_denial_diagnostic)),
    method!("recovery.lifeline_command_execution_observation_denial_diagnostic_selftest", Head, ["recovery.lifeline_command_execution_observation_denial_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_execution_observation_denial_diagnostic_selftest)),
    method!("recovery.lifeline_command_execution_completion_denial_diagnostic", Head, ["recovery.lifeline_command_execution_completion_denial"], [], MethodAction::ReadMethod(emit_recovery_lifeline_command_execution_completion_denial_diagnostic)),
    method!("recovery.lifeline_command_execution_completion_denial_diagnostic_selftest", Head, ["recovery.lifeline_command_execution_completion_denial_selftest"], [], MethodAction::Read0(emit_recovery_lifeline_command_execution_completion_denial_diagnostic_selftest)),
    method!("recovery.lifeline_status_execution_result_diagnostic", Exact, ["recovery.lifeline_status_execution_result"], [], MethodAction::Read0(emit_recovery_lifeline_status_execution_result_diagnostic)),
    envelope_method!("recovery.lifeline_status_result_read", Exact, ["recovery.lifeline.status"], [], 18, "recovery.lifeline.status", "cap.recovery.load_artifact.read", "agent_command_envelope.current_boot.serial.recovery_lifeline_status.v0", "recovery.lifeline.status", MethodAction::StatusResultRead),
    method!("recovery.load_binding", Head, ["module.recovery_load_binding"], [], MethodAction::Read0(emit_recovery_artifact_load_binding)),
    method!("recovery.load_binding_selftest", Head, ["module.recovery_load_binding_selftest"], [], MethodAction::Read0(emit_recovery_artifact_load_binding_selftest)),
    method!("provider.context_export", Head, ["provider.export_context"], [route!("provider.context_export"), route!("provider.export_context")], MethodAction::DeniedProviderContextExport),
    pred_method!("module.load_ephemeral", hello_service::is_load_start_method, [], MethodAction::ResponseMethod(hello_service::emit_load_start)),
    pred_method!("module.load_ephemeral", echo_service::is_load_method, [], MethodAction::ResponseMethod(echo_service::emit_load)),
    pred_method!("module.load_ephemeral", granted_candidate_service::is_load_method, [], MethodAction::ResponseMethod(granted_candidate_service::emit_load)),
    pred_method!("service.start", hello_service::is_start_method, [route!("service.start")], MethodAction::ResponseMethod(hello_service::emit_start)),
    pred_method!("service.start", echo_service::is_start_method, [], MethodAction::ResponseMethod(echo_service::emit_start)),
    pred_method!("service.start", granted_candidate_service::is_start_method, [], MethodAction::ResponseMethod(granted_candidate_service::emit_start)),
    pred_method!("service.restart", hello_service::is_restart_method, [route!("service.restart")], MethodAction::ResponseMethod(hello_service::emit_restart)),
    pred_method!("service.hot_swap", hello_service::is_hot_swap_method, [route!("service.hot_swap")], MethodAction::ResponseMethod(hello_service::emit_hot_swap)),
    pred_envelope_method!("service.rollback_preview", hello_service::is_rollback_preview_method, [route!("service.rollback_preview")], 7, "service.rollback_preview", "cap.service.rollback_preview.read", "agent_command_envelope.current_boot.serial.service_rollback_preview.v0", "service.rollback_preview svc.demo.hello", MethodAction::ResponseMethod(hello_service::emit_rollback_preview)),
    pred_envelope_method!("service.rollback_preview", granted_candidate_service::is_rollback_preview_method, [], 19, "service.rollback_preview", "cap.service.granted_candidate.rollback_preview.read", "agent_command_envelope.current_boot.serial.service_granted_candidate_rollback_preview.v0", "service.rollback_preview svc.dev.granted_candidate", MethodAction::ResponseMethod(granted_candidate_service::emit_rollback_preview)),
    pred_method!("service.rollback_apply", hello_service::is_rollback_apply_method, [route!("service.rollback_apply")], MethodAction::DeniedMethod(hello_service::emit_rollback_apply)),
    pred_method!("service.rollback_apply", granted_candidate_service::is_rollback_apply_method, [], MethodAction::ResponseMethod(granted_candidate_service::emit_rollback_apply)),
    pred_method!("recovery.rollback_inspect_source_reference_selftest", hello_service::is_recovery_rollback_inspect_source_reference_selftest_method, [route!("recovery.rollback_inspect_source_reference_selftest")], MethodAction::Response0Read(hello_service::emit_recovery_rollback_inspect_source_reference_selftest)),
    pred_method!("recovery.rollback_materialize_dry_run", hello_service::is_recovery_rollback_materialize_dry_run_method, [route!("recovery.rollback_materialize_dry_run")], MethodAction::ResponseMaterializeDryRun(hello_service::emit_recovery_rollback_materialize_dry_run)),
    pred_method!("service.stop", hello_service::is_stop_method, [route!("service.stop")], MethodAction::ResponseMethod(hello_service::emit_stop)),
    pred_method!("service.stop", echo_service::is_stop_method, [], MethodAction::ResponseMethod(echo_service::emit_stop)),
    pred_method!("service.stop", granted_candidate_service::is_stop_method, [], MethodAction::ResponseMethod(granted_candidate_service::emit_stop)),
    pred_method!("service.drop", hello_service::is_drop_method, [route!("service.drop")], MethodAction::ResponseMethod(hello_service::emit_drop)),
    pred_method!("service.drop", echo_service::is_drop_method, [], MethodAction::ResponseMethod(echo_service::emit_drop)),
    pred_method!("service.drop", granted_candidate_service::is_drop_method, [], MethodAction::ResponseMethod(granted_candidate_service::emit_drop)),
    method!("module.load_ephemeral", Head, ["service.load_ephemeral"], [route!("module.load_ephemeral"), route!("service.load_ephemeral")], MethodAction::DeniedModuleLoadEphemeral),
    method!("recovery.load_artifact", Exact, ["module.load_recovery_artifact"], [route!("recovery.load_artifact"), route!("module.load_recovery_artifact")], MethodAction::DeniedRecoveryArtifactLoad),
    method!("memory.record_observation", Exact, [], [route!("memory.record_observation" => "memory.record_observation")], MethodAction::DeniedMemoryMutation),
    method!("memory.propose_policy", Exact, [], [route!("memory.propose_policy" => "memory.propose_policy")], MethodAction::DeniedMemoryMutation),
    method!("memory.supersede_fact", Exact, [], [route!("memory.supersede_fact" => "memory.supersede_fact")], MethodAction::DeniedMemoryMutation),
    method!("memory.redact", Exact, [], [route!("memory.redact" => "memory.redact")], MethodAction::DeniedMemoryMutation),
    method!("memory.compact", Exact, [], [route!("memory.compact" => "memory.compact")], MethodAction::DeniedMemoryMutation),
    method!("module.propose", Head, [], [route!("module.propose")], MethodAction::DeniedGeneric),
    method!("module.build_result", Head, [], [route!("module.build_result")], MethodAction::DeniedGeneric),
    method!("module.test_request", Head, [], [route!("module.test_request")], MethodAction::DeniedGeneric),
    method!("module.test_result", Head, [], [route!("module.test_result")], MethodAction::DeniedGeneric),
    method!("module.load_recovery_artifact", Head, [], [], MethodAction::DeniedGeneric),
    method!("recovery.load_artifact", Head, [], [], MethodAction::DeniedGeneric),
    method!("module.persist", Head, [], [route!("module.persist")], MethodAction::DeniedGeneric),
    method!("module.rollback", Head, [], [route!("module.rollback")], MethodAction::DeniedGeneric),
    method!("service.load_ephemeral", Head, [], [], MethodAction::DeniedGeneric),
    method!("service.restart", Head, [], [], MethodAction::DeniedGeneric),
    method!("service.hot_swap", Head, [], [], MethodAction::DeniedGeneric),
    method!("service.rollback_apply", Head, [], [], MethodAction::DeniedGeneric),
    method!("service.health", Head, [], [], MethodAction::DeniedGeneric),
    method!("service.start", Head, [], [], MethodAction::DeniedGeneric),
    method!("service.stop", Head, [], [], MethodAction::DeniedGeneric),
    method!("service.drop", Head, [], [], MethodAction::DeniedGeneric),
    method!("config.apply", Head, [], [route!("config.apply")], MethodAction::DeniedGeneric),
    method!("apply_config", Head, [], [route!("apply_config")], MethodAction::DeniedGeneric),
    method!("provider.configure", Head, [], [], MethodAction::DeniedGeneric),
    method!("wifi.configure", Head, [], [], MethodAction::DeniedGeneric),
    method!("draw_text", Head, [], [], MethodAction::DeniedGeneric),
    method!("probe_device", Head, [], [], MethodAction::DeniedGeneric),
    method!("download_signed_module", Head, [], [], MethodAction::DeniedGeneric),
    method!("run_module_test", Head, [], [], MethodAction::DeniedGeneric),
];

fn lookup_method(method: &str) -> Option<MethodCall<'_>> {
    let mut idx = 0usize;
    while idx < AGENT_METHODS.len() {
        let entry = &AGENT_METHODS[idx];
        if let Some(canonical) = entry_matches(entry, method) {
            return Some(MethodCall {
                input: method,
                canonical,
                entry,
            });
        }
        idx += 1;
    }
    None
}

pub(crate) fn method_registered_exact(method: &str) -> bool {
    lookup_method(method).is_some()
}

fn entry_matches(entry: &'static MethodEntry, method: &str) -> Option<&'static str> {
    match entry.match_kind {
        MatchKind::Exact => {
            if method_eq(method, entry.canonical) || aliases_match_exact(method, entry.aliases) {
                Some(entry.canonical)
            } else {
                None
            }
        }
        MatchKind::Head => {
            if method_head_eq(method, entry.canonical) || aliases_match_head(method, entry.aliases)
            {
                Some(entry.canonical)
            } else {
                None
            }
        }
        MatchKind::Predicate(predicate) => predicate(method).then_some(entry.canonical),
    }
}

fn aliases_match_exact(method: &str, aliases: &[&str]) -> bool {
    let mut idx = 0usize;
    while idx < aliases.len() {
        if method_eq(method, aliases[idx]) {
            return true;
        }
        idx += 1;
    }
    false
}

fn aliases_match_head(method: &str, aliases: &[&str]) -> bool {
    let mut idx = 0usize;
    while idx < aliases.len() {
        if method_head_eq(method, aliases[idx]) {
            return true;
        }
        idx += 1;
    }
    false
}

fn dispatch_method_entry(call: MethodCall<'_>, runtime: ui::RuntimeStatus) -> DispatchOutcome {
    match call.entry.action {
        MethodAction::Read0(emit) => {
            record_read(call.canonical);
            emit();
            DispatchOutcome::Response(call.canonical)
        }
        MethodAction::ReadRuntime(emit) => {
            record_read(call.canonical);
            emit(runtime);
            DispatchOutcome::Response(call.canonical)
        }
        MethodAction::ReadMethod(emit) => {
            record_read(call.canonical);
            emit(call.input);
            DispatchOutcome::Response(call.canonical)
        }
        MethodAction::ReadRuntimeMethod(emit) => {
            record_read(call.canonical);
            emit(runtime, call.input);
            DispatchOutcome::Response(call.canonical)
        }
        MethodAction::ReadRuntimeMethodEvent(emit) => {
            let event_id = record_read(call.canonical);
            emit(runtime, call.input, event_id);
            DispatchOutcome::Response(call.canonical)
        }
        MethodAction::Response0Read(emit) => {
            record_read(call.canonical);
            DispatchOutcome::Response(emit())
        }
        MethodAction::ResponseMethod(emit) => DispatchOutcome::Response(emit(call.input)),
        MethodAction::ResponseMethodReadEvent(emit) => {
            let event_id = record_read(call.canonical);
            DispatchOutcome::Response(emit(call.input, event_id))
        }
        MethodAction::ResponseMaterializeDryRun(emit) => {
            let event_id = event_log::record_hello_recovery_rollback_materialize_dry_run();
            DispatchOutcome::Response(emit(call.input, event_id))
        }
        MethodAction::DeniedMethod(emit) => DispatchOutcome::Denied(emit(call.input)),
        MethodAction::DeniedProviderContextExport => {
            let event_id = record_denial("provider.context_export");
            emit_provider_context_export_denied(runtime, call.input, event_id);
            DispatchOutcome::Denied("provider.context_export")
        }
        MethodAction::DeniedModuleLoadEphemeral => {
            let method = if method_head_eq(call.input, "service.load_ephemeral") {
                "service.load_ephemeral"
            } else {
                "module.load_ephemeral"
            };
            let (event_id, gate_binding) = event_log::record_module_load_ephemeral_denied(method);
            emit_module_load_ephemeral_denied(method, event_id, gate_binding);
            DispatchOutcome::Denied(method)
        }
        MethodAction::DeniedRecoveryArtifactLoad => {
            let method = if method_eq(call.input, "module.load_recovery_artifact") {
                "module.load_recovery_artifact"
            } else {
                "recovery.load_artifact"
            };
            let event_id = event_log::record_recovery_artifact_load_denied(method);
            emit_recovery_artifact_load_denied(method, event_id);
            DispatchOutcome::Denied(method)
        }
        MethodAction::DeniedMemoryMutation => {
            let method = call.canonical;
            let event_id = record_denial(method);
            emit_memory_capability_denied(method, event_id);
            DispatchOutcome::Denied(method)
        }
        MethodAction::DeniedGeneric => {
            let method = call.canonical;
            let event_id = record_denial(method);
            emit_capability_denied(method, event_id);
            DispatchOutcome::Denied(method)
        }
        MethodAction::StatusResultRead => {
            let response_method = if method_eq(call.input, "recovery.lifeline.status") {
                "recovery.lifeline.status"
            } else {
                "recovery.lifeline_status_result_read"
            };
            record_read(response_method);
            emit_recovery_lifeline_status_result_read(response_method);
            DispatchOutcome::Response(response_method)
        }
    }
}

pub(crate) fn console_dispatch_method<'a>(command: &str, input: &'a str) -> Option<&'a str> {
    let mut idx = 0usize;
    while idx < AGENT_METHODS.len() {
        let entry = &AGENT_METHODS[idx];
        let mut route_idx = 0usize;
        while route_idx < entry.console_routes.len() {
            let route = entry.console_routes[route_idx];
            if method_eq(command, route.command) {
                return Some(route.dispatch_method.unwrap_or(input));
            }
            route_idx += 1;
        }
        idx += 1;
    }
    None
}

pub(crate) fn command_envelope_target(value: Option<&str>) -> Option<CommandEnvelopeTarget> {
    let value = value.unwrap_or("");
    let mut idx = 0usize;
    while idx < AGENT_METHODS.len() {
        if let Some(envelope) = AGENT_METHODS[idx].envelope {
            if method_eq(value, envelope.target.method) {
                return Some(envelope.target);
            }
        }
        idx += 1;
    }
    None
}

pub(crate) fn command_envelope_capability(value: Option<&str>) -> Option<&'static str> {
    let value = value.unwrap_or("");
    let mut idx = 0usize;
    while idx < AGENT_METHODS.len() {
        if let Some(envelope) = AGENT_METHODS[idx].envelope {
            if method_eq(value, envelope.target.capability) {
                return Some(envelope.target.capability);
            }
        }
        idx += 1;
    }
    None
}

pub(crate) fn command_envelope_target_at(order: usize) -> Option<CommandEnvelopeTarget> {
    let mut idx = 0usize;
    while idx < AGENT_METHODS.len() {
        if let Some(envelope) = AGENT_METHODS[idx].envelope {
            if envelope.order as usize == order {
                return Some(envelope.target);
            }
        }
        idx += 1;
    }
    None
}

pub(crate) fn command_envelope_target_count() -> usize {
    let mut count = 0usize;
    let mut idx = 0usize;
    while idx < AGENT_METHODS.len() {
        if AGENT_METHODS[idx].envelope.is_some() {
            count += 1;
        }
        idx += 1;
    }
    count
}

pub fn dispatch(method: &str, runtime: ui::RuntimeStatus) -> DispatchOutcome {
    let method = method.trim();
    if method.is_empty() {
        return DispatchOutcome::Unknown;
    }

    // Recovery lifeline is a SEPARATE dispatch path, checked before the general
    // method table so the minimal restore-only surface is provably isolated.
    if let Some(outcome) = recovery_lifeline::dispatch(method, runtime) {
        return outcome;
    }

    if let Some(call) = lookup_method(method) {
        return (call.entry.handler)(call, runtime);
    }

    DispatchOutcome::Unknown
}
