// NET-6: pub(crate) so wasm_runtime's acquire shims can call the ONE shared
// chunk-accept/finalize seam instead of cloning the M12 verifier. Crate-internal
// only. Layering debt noted: the seam's impure staging half still lives in the
// protocol layer; if it grows, move it to a neutral kernel module (or raios-core)
// and have both transports call that instead.
#[path = "agent_protocol_registry.rs"]
pub(crate) mod agent_protocol_registry;
#[path = "artifact_store.rs"]
pub(crate) mod artifact_store;
#[path = "boot_control.rs"]
pub(crate) mod boot_control;
#[path = "distribution_registry.rs"]
pub(crate) mod distribution_registry;
#[path = "durable_store.rs"]
pub(crate) mod durable_store;
#[path = "recovery_lifeline.rs"]
pub(crate) mod recovery_lifeline;
#[path = "repromotion.rs"]
pub(crate) mod repromotion;

use crate::{
    agent_protocol_build_assemble::{
        emit_build_assemble_probe, emit_build_assemble_revision, emit_build_run_prepare,
    },
    agent_protocol_distribution::{
        emit_distribution_provenance_diagnostic, emit_distribution_provenance_diagnostic_selftest,
    },
    agent_protocol_honesty::emit_system_honesty_report,
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
    agent_protocol_program::{
        emit_finalize as emit_program_finalize, emit_rollback_apply as emit_program_rollback_apply,
        emit_rollback_preview as emit_program_rollback_preview,
        emit_submit_chunk as emit_program_submit_chunk, emit_workspace as emit_program_workspace,
    },
    agent_protocol_project::{
        emit_agent_answer_fixture as emit_project_agent_answer_fixture,
        emit_feedback_packet as emit_project_feedback_packet,
        emit_feedback_submit as emit_project_feedback_submit,
        emit_import_begin as emit_project_import_begin,
        emit_import_chunk as emit_project_import_chunk,
        emit_import_commit as emit_project_import_commit,
        emit_import_file_begin as emit_project_import_file_begin,
        emit_import_file_finalize as emit_project_import_file_finalize,
        emit_inspect as emit_project_inspect,
        emit_revision_answer_fixture as emit_project_revision_answer_fixture,
        emit_rwir_answer_fixture as emit_project_rwir_answer_fixture,
        emit_verify_revision as emit_project_verify_revision,
        emit_workspace as emit_project_workspace,
    },
    agent_protocol_project_build::{
        emit_begin as emit_project_build_begin, emit_commit as emit_project_build_commit,
        emit_dependency_read as emit_project_build_dependency_read,
        emit_discard as emit_project_build_discard, emit_receipts as emit_project_build_receipts,
        emit_run as emit_project_build_run, emit_source_read as emit_project_build_source_read,
    },
    agent_protocol_project_dependency::{
        emit_begin as emit_project_dependency_begin, emit_chunk as emit_project_dependency_chunk,
        emit_commit as emit_project_dependency_commit,
        emit_dependencies as emit_project_dependencies,
        emit_discard as emit_project_dependency_discard,
        emit_file_begin as emit_project_dependency_file_begin,
        emit_file_finalize as emit_project_dependency_file_finalize,
    },
    agent_protocol_project_editor::{
        emit_edit_begin as emit_project_edit_begin, emit_edit_chunk as emit_project_edit_chunk,
        emit_edit_commit as emit_project_edit_commit, emit_edit_delete as emit_project_edit_delete,
        emit_edit_diff as emit_project_edit_diff, emit_edit_discard as emit_project_edit_discard,
        emit_edit_file_begin as emit_project_edit_file_begin,
        emit_edit_file_finalize as emit_project_edit_file_finalize,
    },
    agent_protocol_project_install::{
        emit_install_prepare as emit_project_install_prepare,
        emit_install_serial_approval_denied as emit_project_install_approval_denied,
        emit_install_signature as emit_project_install_signature,
        emit_install_status as emit_project_install_status,
        emit_rollback_status as emit_project_rollback_status,
        emit_uninstall_prepare as emit_project_uninstall_prepare,
        emit_uninstall_serial_approval_denied as emit_project_uninstall_approval_denied,
        emit_uninstall_signature as emit_project_uninstall_signature,
    },
    agent_protocol_project_query::{
        emit_read as emit_project_read, emit_search as emit_project_search,
    },
    agent_protocol_project_run::{
        emit_cancel as emit_project_run_cancel, emit_drop as emit_workspace_drop,
        emit_health as emit_workspace_health, emit_prepare as emit_project_run_prepare,
        emit_serial_approval_denied as emit_project_run_approval_denied,
        emit_start as emit_workspace_start, emit_status as emit_project_run_status,
        emit_stop as emit_workspace_stop,
    },
    agent_protocol_provider::{
        emit_provider_context_export_authorized_selftest,
        emit_provider_context_export_authorized_selftest_smuggle,
        emit_provider_context_export_denied, emit_provider_context_export_packet_selftest,
        emit_provider_context_gate, emit_provider_context_gate_selftest,
        emit_provider_context_injection_gate, emit_provider_context_injection_gate_selftest,
        emit_provider_trust_honesty,
    },
    agent_protocol_support::{method_eq, method_head_eq},
    agent_protocol_system::{
        emit_boot_log, emit_capabilities, emit_describe, emit_device_graph, emit_persist_layout,
        emit_problem_list, emit_service_inventory, emit_snapshot,
    },
    agent_protocol_time::{emit_system_cert_time_check_selftest, emit_system_time_authority},
    agent_protocol_ui::emit_personal_shell_proof,
    agent_protocol_wasm::{
        emit_submit_candidate_chunk, emit_submit_candidate_finalize, emit_transport_lease_probe,
        emit_wasm_acquire_import_probe, emit_wasm_acquisition_service_probe,
        emit_wasm_beyond_env_lifecycle_probe, emit_wasm_bufecho_probe, emit_wasm_certspki_probe,
        emit_wasm_certwindow_probe, emit_wasm_crypto_import_probe, emit_wasm_dnsparse_probe,
        emit_wasm_echo_probe, emit_wasm_httphead_probe,
    },
    echo_service, event_log, granted_candidate_service, hello_service, memory_store, ui,
    workspace_candidate_service,
};

use self::agent_protocol_registry::{
    emit_distribution_receiver_identity_load_preflight, emit_registry_selection_diagnostic,
    emit_registry_selection_diagnostic_selftest, emit_submit_distribution_begin,
    emit_submit_distribution_begin_from_catalog, emit_submit_distribution_catalog_entry,
    emit_submit_distribution_chunk, emit_submit_distribution_finalize,
    emit_submit_distribution_receiver_identity,
    emit_submit_distribution_receiver_identity_evidence,
    emit_submit_distribution_receiver_identity_finalize,
};

pub(crate) use self::agent_protocol_registry::DistributionReceiverIdentityLoadPreflightProjection;
pub(crate) use self::repromotion::run_provider_autoload;
pub(crate) use crate::agent_protocol_provider::provider_minimal_context_evidence_for_runtime;

pub(crate) fn receiver_identity_load_preflight_projection(
) -> DistributionReceiverIdentityLoadPreflightProjection {
    agent_protocol_registry::receiver_identity_load_preflight_projection()
}

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
    DeniedMemoryMutation,
    DeniedGeneric,
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
    method!("isolation.selftest", Exact, ["isolation-selftest"], [route!("isolation.selftest")], MethodAction::Read0(crate::wasm_runtime::emit_isolation_selftest)),
    method!("capability.selftest", Exact, ["capability-selftest"], [route!("capability.selftest")], MethodAction::Read0(crate::wasm_runtime::emit_capability_selftest)),
    method!("host_import.selftest", Exact, ["host-import-selftest"], [route!("host_import.selftest")], MethodAction::Read0(crate::wasm_runtime::emit_host_import_selftest)),
    method!("revoke.selftest", Exact, ["revoke-selftest"], [route!("revoke.selftest")], MethodAction::Read0(crate::wasm_runtime::emit_revoke_selftest)),
    method!("storage.selftest", Exact, ["storage-selftest"], [route!("storage.selftest")], MethodAction::Read0(crate::wasm_runtime::emit_storage_selftest)),
    method!("threads.selftest", Exact, ["threads-selftest"], [route!("threads.selftest")], MethodAction::Read0(crate::wasm_runtime::emit_threads_selftest)),
    method!("wasi.selftest", Exact, ["wasi-selftest"], [route!("wasi.selftest")], MethodAction::Read0(crate::wasm_runtime::emit_wasi_selftest)),
    method!("wasi.threadselftest", Exact, ["wasi-threadselftest"], [route!("wasi.threadselftest")], MethodAction::Read0(crate::wasm_runtime::emit_wasi_thread_selftest)),
    method!("wasi.memselftest", Exact, ["wasi-memselftest"], [route!("wasi.memselftest")], MethodAction::Read0(crate::wasm_runtime::emit_wasi_mem_selftest)),
    method!("wasi.sysimport", Exact, ["wasi-sysimport"], [route!("wasi.sysimport")], MethodAction::Read0(crate::wasm_runtime::emit_wasi_sysimport)),
    method!("wasi.compilerload", Exact, ["wasi-compilerload"], [route!("wasi.compilerload")], MethodAction::Read0(crate::wasm_runtime::emit_wasi_compilerload)),
    method!("wasi.rustcrun", Exact, ["wasi-rustcrun"], [route!("wasi.rustcrun")], MethodAction::Read0(crate::wasm_runtime::emit_wasi_rustcrun)),
    method!("wasi.rustcbuild", Exact, ["wasi-rustcbuild"], [route!("wasi.rustcbuild")], MethodAction::Read0(crate::wasm_runtime::emit_wasi_rustcbuild)),
    method!("wasi.rustcdiag", Exact, ["wasi-rustcdiag"], [route!("wasi.rustcdiag")], MethodAction::Read0(crate::wasm_runtime::emit_wasi_rustcdiag)),
    method!("wasi.rustclock", Exact, ["wasi-rustclock"], [route!("wasi.rustclock")], MethodAction::Read0(crate::wasm_runtime::emit_wasi_rustclock)),
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
    pred_method!("service.health", workspace_candidate_service::is_health_method, [], MethodAction::ResponseMethod(emit_workspace_health)),
    method!("memory.profile", Exact, ["memprofile"], [route!("memory.profile" => "memory.profile"), route!("memprofile" => "memory.profile")], MethodAction::Read0(emit_memory_profile)),
    method!("memory.context", Head, ["memctx"], [route!("memory.context"), route!("memctx")], MethodAction::ReadRuntimeMethodEvent(emit_memory_context)),
    method!("memory.query", Head, ["memquery"], [route!("memory.query"), route!("memquery")], MethodAction::Read0(emit_memory_query)),
    method!("memory.trace", Head, ["memtrace"], [route!("memory.trace"), route!("memtrace")], MethodAction::ReadMethod(emit_memory_trace)),
    method!("memory.recent_events", Head, ["audit.events", "events"], [route!("memory.recent_events"), route!("audit.events"), route!("events")], MethodAction::ReadMethod(emit_recent_events)),
    method!("provider.context_gate", Head, ["provider.context_export_status"], [route!("provider.context_gate"), route!("provider.context_export_status")], MethodAction::ReadRuntimeMethod(emit_provider_context_gate)),
    method!("provider.context_gate_selftest", Head, [], [route!("provider.context_gate_selftest")], MethodAction::ReadRuntimeMethod(emit_provider_context_gate_selftest)),
    method!("provider.context_injection_gate", Head, [], [route!("provider.context_injection_gate")], MethodAction::ReadRuntimeMethod(emit_provider_context_injection_gate)),
    method!("provider.context_injection_gate_selftest", Head, [], [route!("provider.context_injection_gate_selftest")], MethodAction::ReadRuntimeMethod(emit_provider_context_injection_gate_selftest)),
    method!("provider.trust_honesty", Head, [], [route!("provider.trust_honesty")], MethodAction::ReadMethod(emit_provider_trust_honesty)),
    method!("system.time_authority", Head, ["time.authority"], [route!("system.time_authority"), route!("time.authority" => "system.time_authority")], MethodAction::ReadMethod(emit_system_time_authority)),
    method!("system.cert_time_check_selftest", Head, ["cert.time_check_selftest"], [route!("system.cert_time_check_selftest"), route!("cert.time_check_selftest" => "system.cert_time_check_selftest")], MethodAction::ReadMethod(emit_system_cert_time_check_selftest)),
    method!("system.honesty_report", Head, ["honesty.report"], [route!("system.honesty_report"), route!("honesty.report" => "system.honesty_report")], MethodAction::ReadMethod(emit_system_honesty_report)),
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
    method!("wasm.bufecho_probe", Exact, [], [route!("wasm.bufecho_probe")], MethodAction::Read0(emit_wasm_bufecho_probe)),
    method!("wasm.certwindow_probe", Exact, [], [route!("wasm.certwindow_probe")], MethodAction::Read0(emit_wasm_certwindow_probe)),
    method!("wasm.httphead_probe", Exact, [], [route!("wasm.httphead_probe")], MethodAction::Read0(emit_wasm_httphead_probe)),
    method!("wasm.certspki_probe", Exact, [], [route!("wasm.certspki_probe")], MethodAction::Read0(emit_wasm_certspki_probe)),
    method!("wasm.dnsparse_probe", Exact, [], [route!("wasm.dnsparse_probe")], MethodAction::Read0(emit_wasm_dnsparse_probe)),
    method!("build.assemble_probe", Exact, [], [route!("build.assemble_probe")], MethodAction::Read0(emit_build_assemble_probe)),
    method!("build.assemble_revision", Exact, [], [route!("build.assemble_revision")], MethodAction::Read0(emit_build_assemble_revision)),
    method!("build.run_prepare", Exact, [], [route!("build.run_prepare")], MethodAction::Read0(emit_build_run_prepare)),
    method!("wasm.beyond_env_lifecycle_probe", Head, [], [route!("wasm.beyond_env_lifecycle_probe")], MethodAction::ReadMethod(emit_wasm_beyond_env_lifecycle_probe)),
    method!("wasm.crypto_import_probe", Exact, [], [route!("wasm.crypto_import_probe")], MethodAction::Read0(emit_wasm_crypto_import_probe)),
    method!("wasm.acquire_import_probe", Exact, [], [route!("wasm.acquire_import_probe")], MethodAction::Read0(emit_wasm_acquire_import_probe)),
    method!("wasm.acquisition_service_probe", Exact, [], [route!("wasm.acquisition_service_probe")], MethodAction::Read0(emit_wasm_acquisition_service_probe)),
    method!("network.transport_lease_probe", Head, [], [route!("network.transport_lease_probe")], MethodAction::ReadMethod(emit_transport_lease_probe)),
    method!("ui.personal_shell_proof", Head, [], [route!("ui.personal_shell_proof")], MethodAction::ReadMethod(emit_personal_shell_proof)),
    method!("echo.invoke_fuel_starved", Exact, [], [route!("echo.invoke_fuel_starved")], MethodAction::Read0(echo_service::emit_invoke_fuel_starved)),
    method!("module.submit_candidate_chunk", Head, [], [route!("module.submit_candidate_chunk")], MethodAction::ReadMethod(emit_submit_candidate_chunk)),
    method!("module.submit_candidate_finalize", Exact, [], [route!("module.submit_candidate_finalize")], MethodAction::Read0(emit_submit_candidate_finalize)),
    method!("program.submit_chunk", Head, [], [route!("program.submit_chunk")], MethodAction::ReadMethod(emit_program_submit_chunk)),
    method!("program.submit_finalize", Exact, [], [route!("program.submit_finalize")], MethodAction::Read0(emit_program_finalize)),
    method!("program.workspace", Exact, [], [route!("program.workspace")], MethodAction::Read0(emit_program_workspace)),
    method!("program.rollback_preview", Head, [], [route!("program.rollback_preview")], MethodAction::ReadMethod(emit_program_rollback_preview)),
    method!("program.rollback_apply", Head, [], [route!("program.rollback_apply")], MethodAction::ReadRuntimeMethodEvent(emit_program_rollback_apply)),
    method!("project.agent_answer_fixture", Exact, [], [route!("project.agent_answer_fixture")], MethodAction::Read0(emit_project_agent_answer_fixture)),
    method!("project.rwir_answer_fixture", Exact, [], [route!("project.rwir_answer_fixture")], MethodAction::Read0(emit_project_rwir_answer_fixture)),
    method!("project.verify_revision", Exact, [], [route!("project.verify_revision")], MethodAction::Read0(emit_project_verify_revision)),
    method!("project.feedback_packet", Exact, [], [route!("project.feedback_packet")], MethodAction::Read0(emit_project_feedback_packet)),
    method!("project.feedback_submit", Exact, [], [route!("project.feedback_submit")], MethodAction::ReadRuntime(emit_project_feedback_submit)),
    method!("project.revision_answer_fixture", Exact, [], [route!("project.revision_answer_fixture")], MethodAction::Read0(emit_project_revision_answer_fixture)),
    method!("project.workspace", Exact, [], [route!("project.workspace")], MethodAction::Read0(emit_project_workspace)),
    method!("project.import_begin", Head, [], [route!("project.import_begin")], MethodAction::ReadMethod(emit_project_import_begin)),
    method!("project.import_file_begin", Head, [], [route!("project.import_file_begin")], MethodAction::ReadMethod(emit_project_import_file_begin)),
    method!("project.import_chunk", Head, [], [route!("project.import_chunk")], MethodAction::ReadMethod(emit_project_import_chunk)),
    method!("project.import_file_finalize", Exact, [], [route!("project.import_file_finalize")], MethodAction::Read0(emit_project_import_file_finalize)),
    method!("project.import_commit", Exact, [], [route!("project.import_commit")], MethodAction::Read0(emit_project_import_commit)),
    method!("project.inspect", Head, [], [route!("project.inspect")], MethodAction::ReadMethod(emit_project_inspect)),
    method!("project.read", Head, [], [route!("project.read")], MethodAction::ReadMethod(emit_project_read)),
    method!("project.search", Head, [], [route!("project.search")], MethodAction::ReadMethod(emit_project_search)),
    method!("project.edit_begin", Head, [], [route!("project.edit_begin")], MethodAction::ReadMethod(emit_project_edit_begin)),
    method!("project.edit_file_begin", Head, [], [route!("project.edit_file_begin")], MethodAction::ReadMethod(emit_project_edit_file_begin)),
    method!("project.edit_chunk", Head, [], [route!("project.edit_chunk")], MethodAction::ReadMethod(emit_project_edit_chunk)),
    method!("project.edit_file_finalize", Exact, [], [route!("project.edit_file_finalize")], MethodAction::Read0(emit_project_edit_file_finalize)),
    method!("project.edit_delete", Head, [], [route!("project.edit_delete")], MethodAction::ReadMethod(emit_project_edit_delete)),
    method!("project.edit_diff", Exact, [], [route!("project.edit_diff")], MethodAction::Read0(emit_project_edit_diff)),
    method!("project.edit_commit", Exact, [], [route!("project.edit_commit")], MethodAction::Read0(emit_project_edit_commit)),
    method!("project.edit_discard", Exact, [], [route!("project.edit_discard")], MethodAction::Read0(emit_project_edit_discard)),
    method!("project.dependency_begin", Head, [], [route!("project.dependency_begin")], MethodAction::ReadMethod(emit_project_dependency_begin)),
    method!("project.dependency_file_begin", Head, [], [route!("project.dependency_file_begin")], MethodAction::ReadMethod(emit_project_dependency_file_begin)),
    method!("project.dependency_chunk", Head, [], [route!("project.dependency_chunk")], MethodAction::ReadMethod(emit_project_dependency_chunk)),
    method!("project.dependency_file_finalize", Exact, [], [route!("project.dependency_file_finalize")], MethodAction::Read0(emit_project_dependency_file_finalize)),
    method!("project.dependency_commit", Exact, [], [route!("project.dependency_commit")], MethodAction::Read0(emit_project_dependency_commit)),
    method!("project.dependency_discard", Exact, [], [route!("project.dependency_discard")], MethodAction::Read0(emit_project_dependency_discard)),
    method!("project.build_begin", Head, [], [route!("project.build_begin")], MethodAction::ReadMethod(emit_project_build_begin)),
    method!("project.build_source_read", Head, [], [route!("project.build_source_read")], MethodAction::ReadMethod(emit_project_build_source_read)),
    method!("project.build_dependency_read", Head, [], [route!("project.build_dependency_read")], MethodAction::ReadMethod(emit_project_build_dependency_read)),
    method!("project.build_run", Head, [], [route!("project.build_run")], MethodAction::ReadMethod(emit_project_build_run)),
    method!("project.build_commit", Exact, [], [route!("project.build_commit")], MethodAction::Read0(emit_project_build_commit)),
    method!("project.build_discard", Exact, [], [route!("project.build_discard")], MethodAction::Read0(emit_project_build_discard)),
    method!("project.build_receipts", Head, [], [route!("project.build_receipts")], MethodAction::ReadMethod(emit_project_build_receipts)),
    method!("project.run_prepare", Head, [], [route!("project.run_prepare")], MethodAction::ReadMethod(emit_project_run_prepare)),
    method!("project.run_status", Exact, [], [route!("project.run_status")], MethodAction::Read0(emit_project_run_status)),
    method!("project.run_cancel", Exact, [], [route!("project.run_cancel")], MethodAction::Read0(emit_project_run_cancel)),
    method!("project.run_approve", Head, [], [route!("project.run_approve")], MethodAction::ReadMethod(emit_project_run_approval_denied)),
    method!("project.install_prepare", Head, [], [route!("project.install_prepare")], MethodAction::ReadMethod(emit_project_install_prepare)),
    method!("project.install_signature", Head, [], [route!("project.install_signature")], MethodAction::ReadMethod(emit_project_install_signature)),
    method!("project.install_status", Exact, [], [route!("project.install_status")], MethodAction::Read0(emit_project_install_status)),
    method!("project.install_approve", Head, [], [route!("project.install_approve")], MethodAction::ReadMethod(emit_project_install_approval_denied)),
    method!("project.uninstall_prepare", Head, [], [route!("project.uninstall_prepare")], MethodAction::ReadMethod(emit_project_uninstall_prepare)),
    method!("project.uninstall_signature", Head, [], [route!("project.uninstall_signature")], MethodAction::ReadMethod(emit_project_uninstall_signature)),
    method!("project.uninstall_approve", Head, [], [route!("project.uninstall_approve")], MethodAction::ReadMethod(emit_project_uninstall_approval_denied)),
    method!("project.rollback_status", Exact, [], [route!("project.rollback_status")], MethodAction::Read0(emit_project_rollback_status)),
    method!("project.dependencies", Head, [], [route!("project.dependencies")], MethodAction::ReadMethod(emit_project_dependencies)),
    method!("module.submit_distribution_catalog_entry", Head, [], [route!("module.submit_distribution_catalog_entry")], MethodAction::ReadMethod(emit_submit_distribution_catalog_entry)),
    method!("module.submit_distribution_receiver_identity", Head, [], [route!("module.submit_distribution_receiver_identity")], MethodAction::ReadMethod(emit_submit_distribution_receiver_identity)),
    method!("module.submit_distribution_receiver_identity_evidence", Head, [], [route!("module.submit_distribution_receiver_identity_evidence")], MethodAction::ReadMethod(emit_submit_distribution_receiver_identity_evidence)),
    method!("module.submit_distribution_receiver_identity_finalize", Head, [], [route!("module.submit_distribution_receiver_identity_finalize")], MethodAction::ReadMethod(emit_submit_distribution_receiver_identity_finalize)),
    method!("module.distribution_receiver_identity_load_preflight", Head, [], [route!("module.distribution_receiver_identity_load_preflight")], MethodAction::ReadMethod(emit_distribution_receiver_identity_load_preflight)),
    method!("module.submit_distribution_begin", Head, [], [route!("module.submit_distribution_begin")], MethodAction::ReadMethod(emit_submit_distribution_begin)),
    method!("module.submit_distribution_begin_from_catalog", Head, [], [route!("module.submit_distribution_begin_from_catalog")], MethodAction::ReadMethod(emit_submit_distribution_begin_from_catalog)),
    method!("module.submit_distribution_chunk", Head, [], [route!("module.submit_distribution_chunk")], MethodAction::ReadMethod(emit_submit_distribution_chunk)),
    method!("module.submit_distribution_finalize", Exact, [], [route!("module.submit_distribution_finalize")], MethodAction::Read0(emit_submit_distribution_finalize)),
    method!("module.distribution_provenance_diagnostic", Head, [], [route!("module.distribution_provenance_diagnostic")], MethodAction::ReadMethod(emit_distribution_provenance_diagnostic)),
    method!("module.distribution_provenance_diagnostic_selftest", Head, [], [route!("module.distribution_provenance_diagnostic_selftest")], MethodAction::Read0(emit_distribution_provenance_diagnostic_selftest)),
    method!("module.registry_selection_diagnostic", Head, [], [route!("module.registry_selection_diagnostic")], MethodAction::ReadMethod(emit_registry_selection_diagnostic)),
    method!("module.registry_selection_diagnostic_selftest", Exact, [], [route!("module.registry_selection_diagnostic_selftest")], MethodAction::Read0(emit_registry_selection_diagnostic_selftest)),
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
    pred_envelope_method!("recovery.rollback_inspect", hello_service::is_recovery_rollback_inspect_method, [route!("recovery.rollback_inspect")], 8, "recovery.rollback_inspect", "cap.recovery.rollback_inspect.read", "agent_command_envelope.current_boot.serial.recovery_rollback_inspect.v0", "recovery.rollback_inspect svc.demo.hello", MethodAction::ResponseMethodReadEvent(hello_service::emit_recovery_rollback_inspect)),
    method!("recovery.disable_module_selftest", Exact, [], [route!("recovery.disable_module_selftest")], MethodAction::Read0(durable_store::emit_recovery_action_selftest)),
    method!("recovery.load_artifact_by_hash_selftest", Exact, [], [route!("recovery.load_artifact_by_hash_selftest")], MethodAction::Read0(recovery_lifeline::emit_load_artifact_by_hash_selftest)),
    method!("provider.context_export", Head, ["provider.export_context"], [route!("provider.context_export"), route!("provider.export_context")], MethodAction::DeniedProviderContextExport),
    pred_method!("module.load_ephemeral", hello_service::is_load_start_method, [], MethodAction::ResponseMethod(hello_service::emit_load_start)),
    pred_method!("module.load_ephemeral", echo_service::is_load_method, [], MethodAction::ResponseMethod(echo_service::emit_load)),
    pred_method!("module.load_ephemeral", granted_candidate_service::is_load_method, [], MethodAction::ResponseMethod(granted_candidate_service::emit_load)),
    pred_method!("service.start", hello_service::is_start_method, [route!("service.start")], MethodAction::ResponseMethod(hello_service::emit_start)),
    pred_method!("service.start", echo_service::is_start_method, [], MethodAction::ResponseMethod(echo_service::emit_start)),
    pred_method!("service.start", granted_candidate_service::is_start_method, [], MethodAction::ResponseMethod(granted_candidate_service::emit_start)),
    pred_method!("service.start", workspace_candidate_service::is_start_method, [], MethodAction::ResponseMethod(emit_workspace_start)),
    pred_method!("service.restart", hello_service::is_restart_method, [route!("service.restart")], MethodAction::ResponseMethod(hello_service::emit_restart)),
    pred_method!("service.hot_swap", hello_service::is_hot_swap_method, [route!("service.hot_swap")], MethodAction::ResponseMethod(hello_service::emit_hot_swap)),
    pred_envelope_method!("service.rollback_preview", hello_service::is_rollback_preview_method, [route!("service.rollback_preview")], 7, "service.rollback_preview", "cap.service.rollback_preview.read", "agent_command_envelope.current_boot.serial.service_rollback_preview.v0", "service.rollback_preview svc.demo.hello", MethodAction::ResponseMethod(hello_service::emit_rollback_preview)),
    pred_envelope_method!("service.rollback_preview", granted_candidate_service::is_rollback_preview_method, [], 18, "service.rollback_preview", "cap.service.granted_candidate.rollback_preview.read", "agent_command_envelope.current_boot.serial.service_granted_candidate_rollback_preview.v0", "service.rollback_preview svc.dev.granted_candidate", MethodAction::ResponseMethod(granted_candidate_service::emit_rollback_preview)),
    pred_method!("service.rollback_apply", hello_service::is_rollback_apply_method, [route!("service.rollback_apply")], MethodAction::DeniedMethod(hello_service::emit_rollback_apply)),
    pred_method!("service.rollback_apply", granted_candidate_service::is_rollback_apply_method, [], MethodAction::ResponseMethod(granted_candidate_service::emit_rollback_apply)),
    pred_method!("recovery.rollback_inspect_source_reference_selftest", hello_service::is_recovery_rollback_inspect_source_reference_selftest_method, [route!("recovery.rollback_inspect_source_reference_selftest")], MethodAction::Response0Read(hello_service::emit_recovery_rollback_inspect_source_reference_selftest)),
    pred_method!("recovery.rollback_materialize_dry_run", hello_service::is_recovery_rollback_materialize_dry_run_method, [route!("recovery.rollback_materialize_dry_run")], MethodAction::ResponseMaterializeDryRun(hello_service::emit_recovery_rollback_materialize_dry_run)),
    pred_method!("service.stop", hello_service::is_stop_method, [route!("service.stop")], MethodAction::ResponseMethod(hello_service::emit_stop)),
    pred_method!("service.stop", echo_service::is_stop_method, [], MethodAction::ResponseMethod(echo_service::emit_stop)),
    pred_method!("service.stop", granted_candidate_service::is_stop_method, [], MethodAction::ResponseMethod(granted_candidate_service::emit_stop)),
    pred_method!("service.stop", workspace_candidate_service::is_stop_method, [], MethodAction::ResponseMethod(emit_workspace_stop)),
    pred_method!("service.drop", hello_service::is_drop_method, [route!("service.drop")], MethodAction::ResponseMethod(hello_service::emit_drop)),
    pred_method!("service.drop", echo_service::is_drop_method, [], MethodAction::ResponseMethod(echo_service::emit_drop)),
    pred_method!("service.drop", granted_candidate_service::is_drop_method, [], MethodAction::ResponseMethod(granted_candidate_service::emit_drop)),
    pred_method!("service.drop", workspace_candidate_service::is_drop_method, [], MethodAction::ResponseMethod(emit_workspace_drop)),
    method!("module.load_ephemeral", Head, ["service.load_ephemeral"], [route!("module.load_ephemeral"), route!("service.load_ephemeral")], MethodAction::DeniedModuleLoadEphemeral),
    method!("memory.record_observation", Exact, [], [route!("memory.record_observation" => "memory.record_observation")], MethodAction::DeniedMemoryMutation),
    method!("memory.propose_policy", Exact, [], [route!("memory.propose_policy" => "memory.propose_policy")], MethodAction::DeniedMemoryMutation),
    method!("memory.supersede_fact", Exact, [], [route!("memory.supersede_fact" => "memory.supersede_fact")], MethodAction::DeniedMemoryMutation),
    method!("memory.redact", Exact, [], [route!("memory.redact" => "memory.redact")], MethodAction::DeniedMemoryMutation),
    method!("memory.compact", Exact, [], [route!("memory.compact" => "memory.compact")], MethodAction::DeniedMemoryMutation),
    method!("module.propose", Head, [], [route!("module.propose")], MethodAction::DeniedGeneric),
    method!("module.build_result", Head, [], [route!("module.build_result")], MethodAction::DeniedGeneric),
    method!("module.test_request", Head, [], [route!("module.test_request")], MethodAction::DeniedGeneric),
    method!("module.test_result", Head, [], [route!("module.test_result")], MethodAction::DeniedGeneric),
    method!("module.load_recovery_artifact", Head, [], [route!("module.load_recovery_artifact")], MethodAction::DeniedGeneric),
    method!("recovery.load_artifact", Head, [], [route!("recovery.load_artifact")], MethodAction::DeniedGeneric),
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
