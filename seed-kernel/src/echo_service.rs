use crate::current_boot_service::ServiceDescriptor;

include!(concat!(
    env!("OUT_DIR"),
    "/echo_current_boot_load_descriptor.rs"
));

pub(crate) const ECHO_SERVICE_DESCRIPTOR: ServiceDescriptor = ServiceDescriptor {
    service_id: "svc.demo.echo",
    artifact_id: "wasm:svc.demo.echo",
    artifact_kind: "wasm32_unknown_unknown_service_module",
    scope: "current_boot",
    classification: "local_only",
    persistence: "none",
    service_capability: "cap.service.echo_demo.current_boot",
    health_capability: "cap.service.health.read",
    rollback_preview_capability: "cap.service.echo.rollback_preview.read",
    rollback_apply_capability: "cap.service.echo.rollback_apply.current_boot",
    rollback_materialize_capability: "cap.recovery.echo.rollback_materialize_dry_run.current_boot",
    rollback_inspect_capability: "cap.recovery.echo.rollback_inspect.read",
    primary_alias: "echo",
    host_bound_alias: "host_bound:svc.demo.echo",
    replacement_service_id: "svc.demo.echo.v2",
    replacement_alias: "echo.v2",
    replacement_artifact_identity_id: "builtin_artifact_identity.svc.demo.echo.wasm.v2",
    reset_state_service_id: "svc.demo.echo.reset_state",
    reset_state_alias: "echo.reset_state",
    artifact_load_plan_preflight_id: "artifact_load_plan_preflight.current_boot.svc.demo.echo.v0",
    artifact_load_plan_preflight_status: "accepted_wasm_current_boot_only",
    service_slot_intent_id: "service_slot_intent.current_boot.svc.demo.echo.v0",
    ram_only_service_slot_id: "ram_only:svc.demo.echo",
    service_slot_activation_id: "service_slot_activation.current_boot.svc.demo.echo.v0",
    service_slot_activation_active_status: "active_current_boot",
    service_slot_activation_stopped_status: "stopped_current_boot",
    service_slot_activation_cleared_status: "cleared_current_boot",
    service_slot_activation_missing_status: "missing_current_boot",
    inventory_kind: "service",
    inventory_replaceable: true,
    inventory_core_owned: false,
    inventory_health_running: "healthy",
    inventory_health_stopped: "stopped",
    inventory_health_missing: "missing",
    event_lifecycle_kind: "raios.ram_only_echo_service.lifecycle",
    event_health_kind: "raios.ram_only_echo_service.health",
    event_rollback_preview_kind: "raios.ram_only_echo_service.rollback_preview",
    event_rollback_apply_kind: "raios.ram_only_echo_service.rollback_apply",
};

#[used]
static ECHO_SERVICE_DESCRIPTOR_PROOF: fn() -> bool = validate_echo_service_descriptor;

pub(crate) fn validate_echo_service_descriptor() -> bool {
    let descriptor = ECHO_SERVICE_DESCRIPTOR;
    descriptor.service_id == ECHO_LOAD_DESCRIPTOR_SERVICE_ID
        && descriptor.artifact_id == ECHO_LOAD_DESCRIPTOR_ARTIFACT_ID
        && descriptor.artifact_kind == ECHO_LOAD_DESCRIPTOR_ARTIFACT_KIND
        && descriptor.scope == "current_boot"
        && descriptor.classification == "local_only"
        && descriptor.persistence == "none"
        && descriptor.service_capability == ECHO_LOAD_DESCRIPTOR_SERVICE_CAPABILITY
        && descriptor.ram_only_service_slot_id == ECHO_LOAD_DESCRIPTOR_SERVICE_SLOT_ID
        && ECHO_LOAD_DESCRIPTOR_ARTIFACT_IDENTITY_ID
            == "builtin_artifact_identity.svc.demo.echo.wasm.v0"
        && ECHO_LOAD_DESCRIPTOR_ARTIFACT_IDENTITY_HASH
            == [
                0xb3, 0xf7, 0xf8, 0x5a, 0xf2, 0x25, 0xb9, 0x1a, 0x31, 0x53, 0xc4, 0xc5, 0xdd, 0xd3,
                0x61, 0x4f, 0x36, 0x9d, 0x58, 0x30, 0x45, 0x8c, 0xaa, 0x91, 0xc4, 0x25, 0x31, 0xb1,
                0x99, 0x9f, 0x86, 0xad,
            ]
        && ECHO_LOAD_DESCRIPTOR_ARTIFACT_BYTES_HASH
            == [
                0xf8, 0x1f, 0x94, 0x42, 0xde, 0x37, 0x29, 0xf5, 0x8f, 0x9d, 0x5c, 0x43, 0xb1, 0x86,
                0xa4, 0x22, 0x3e, 0x3f, 0x0e, 0xd0, 0xbd, 0xde, 0x20, 0xe9, 0x47, 0x22, 0xda, 0x8d,
                0x57, 0x33, 0xab, 0xd2,
            ]
        && ECHO_LOAD_DESCRIPTOR_AUTHORIZED_HOST_IMPORTS == "env.log,env.counter_get"
        && ECHO_LOAD_DESCRIPTOR_AUTHORIZED_HOST_IMPORT_COUNT == 2
        && ECHO_LOAD_DESCRIPTOR_AUTHORIZES_CURRENT_BOOT_WASM_EXECUTION
        && ECHO_LOAD_DESCRIPTOR_VALIDATES_WITH_WASMI_MODULE_NEW
        && !ECHO_LOAD_DESCRIPTOR_ACCEPTS_EXTERNAL_ARTIFACT_BYTES
        && !ECHO_LOAD_DESCRIPTOR_LOADS_EXTERNAL_ARTIFACT
        && !ECHO_LOAD_DESCRIPTOR_MAPS_EXECUTABLE_PAGES
        && !ECHO_LOAD_DESCRIPTOR_WRITES_PERSISTENT_STATE
        && !ECHO_LOAD_DESCRIPTOR_AUTHORIZES_PERSISTENT_INSTALL
        && !ECHO_LOAD_DESCRIPTOR_AUTHORIZES_ROLLBACK_INSTALL
        && raios_core::sha256_bytes(ECHO_LOAD_DESCRIPTOR_SOURCE.as_bytes())
            == ECHO_LOAD_DESCRIPTOR_HASH
        && raios_core::sha256_bytes(ECHO_LOAD_DESCRIPTOR_SIGNATURE_ENVELOPE_TEXT.as_bytes())
            == ECHO_LOAD_DESCRIPTOR_SIGNATURE_ENVELOPE_HASH
}
