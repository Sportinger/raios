use super::*;

// (module
//   (func (export "raios_workspace_main") (result i32)
//     i32.const 0))
const ZERO_IMPORT_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x03,
    0x02, 0x01, 0x00, 0x07, 0x18, 0x01, 0x14, 0x72, 0x61, 0x69, 0x6f, 0x73, 0x5f, 0x77, 0x6f, 0x72,
    0x6b, 0x73, 0x70, 0x61, 0x63, 0x65, 0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x0a, 0x06, 0x01,
    0x04, 0x00, 0x41, 0x00, 0x0b,
];

// (module
//   (import "env" "counter_get" (func $counter_get (result i64)))
//   (func $start (call $counter_get) drop)
//   (start $start))
const ZERO_GRANT_HOST_CALL_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x00, 0x01, 0x7e, 0x60,
    0x00, 0x00, 0x02, 0x13, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x0b, 0x63, 0x6f, 0x75, 0x6e, 0x74, 0x65,
    0x72, 0x5f, 0x67, 0x65, 0x74, 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x08, 0x01, 0x01, 0x0a, 0x07,
    0x01, 0x05, 0x00, 0x10, 0x00, 0x1a, 0x0b,
];

// (module
//   (import "service" "start" (func $service_start))
//   (func $start (call $service_start))
//   (start $start))
const CROSS_SERVICE_START_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00, 0x02, 0x11,
    0x01, 0x07, 0x73, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x05, 0x73, 0x74, 0x61, 0x72, 0x74, 0x00,
    0x00, 0x03, 0x02, 0x01, 0x00, 0x08, 0x01, 0x01, 0x0a, 0x06, 0x01, 0x04, 0x00, 0x10, 0x00, 0x0b,
];

#[derive(Clone, Copy)]
struct RefusalEvidence {
    refused_before_instantiation: bool,
    refusal_recorded: bool,
    execution_effect_free: bool,
}

impl RefusalEvidence {
    fn token(self) -> &'static str {
        if self.refused_before_instantiation {
            "refused"
        } else {
            "failed"
        }
    }
}

fn fixture_has_only_import(bytes: &[u8], expected_module: &str, expected_name: &str) -> bool {
    let engine = envelope::metered_engine();
    let Ok(module) = Module::new(&engine, bytes) else {
        return false;
    };
    let mut imports = module.imports();
    imports
        .next()
        .is_some_and(|import| import.module() == expected_module && import.name() == expected_name)
        && imports.next().is_none()
}

fn run_zero_import_fixture() -> bool {
    let run = envelope::execute_workspace_no_import_candidate(ZERO_IMPORT_WASM);
    run.validation_ok
        && run.instantiation_ok
        && run.run_outcome == "success"
        && run.return_value == Some(0)
        && run.import_grant_performed
        && run.import_grant_status == "import_grant_authorized"
        && run.import_grant_reason == "authorized_observed_empty_import_surface"
        && run.authorized_import_count == 0
        && run.linked_host_import_count == 0
        && run.module_imports_within_authorized_list
        && run.missing_import_module.is_none()
        && run.missing_import_name.is_none()
        && run.log_line.is_none()
        && run.captured_output_len == 0
        && run.raw_captured_output.is_empty()
}

fn run_zero_grant_host_call_fixture() -> RefusalEvidence {
    let exact_fixture_import =
        fixture_has_only_import(ZERO_GRANT_HOST_CALL_WASM, "env", "counter_get");
    let run = envelope::execute_workspace_no_import_candidate(ZERO_GRANT_HOST_CALL_WASM);

    // execute_workspace_no_import_candidate rejects at its observed-import
    // preflight, before it constructs the Store, empty Linker, or instance.
    let execution_effect_free = !run.instantiation_ok
        && run.linked_host_import_count == 0
        && run.return_value.is_none()
        && run.fuel_used == 0
        && run.log_line.is_none()
        && run.captured_output_len == 0
        && run.raw_captured_output.is_empty();
    let refused_before_instantiation = exact_fixture_import
        && run.validation_ok
        && run.import_grant_performed
        && run.import_grant_status == "import_grant_authorized"
        && run.import_grant_reason == "authorized_observed_empty_import_surface"
        && run.authorized_import_count == 0
        && !run.module_imports_within_authorized_list
        && run.run_outcome == "workspace_import_surface_not_empty"
        && run.missing_import_module.is_none()
        && run.missing_import_name.is_none()
        && execution_effect_free;

    RefusalEvidence {
        refused_before_instantiation,
        // positive_run emits the import-grant record before returning this
        // machine-readable refusal evidence.
        refusal_recorded: refused_before_instantiation,
        execution_effect_free,
    }
}

fn run_cross_service_fixture() -> RefusalEvidence {
    let exact_fixture_import =
        fixture_has_only_import(CROSS_SERVICE_START_WASM, "service", "start");
    let run = envelope::execute_module_bytes(
        CROSS_SERVICE_START_WASM,
        WORKSPACE_ENTRYPOINT,
        WORKSPACE_SERVICE_ID,
        true,
        &[("service", "start")],
    );

    // service.start is absent from the import vocabulary. Authorization
    // returns unknown_host_import before module compilation, Store/Linker
    // construction, or instantiation.
    let execution_effect_free = !run.instantiation_ok
        && run.linked_host_import_count == 0
        && run.return_value.is_none()
        && run.fuel_used == 0
        && run.log_line.is_none()
        && run.captured_output_len == 0
        && run.raw_captured_output.is_empty();
    let refused_before_instantiation = exact_fixture_import
        && !run.validation_ok
        && !run.import_grant_performed
        && run.import_grant_status == "denied"
        && run.import_grant_reason == "unknown_host_import"
        && run.authorized_import_count == 0
        && !run.module_imports_within_authorized_list
        && run.run_outcome == "import_grant_denied"
        && run.missing_import_module.is_none()
        && run.missing_import_name.is_none()
        && execution_effect_free;

    RefusalEvidence {
        refused_before_instantiation,
        refusal_recorded: refused_before_instantiation,
        execution_effect_free,
    }
}

fn inventory_push_bool(bytes: &mut Vec<u8>, value: bool) {
    bytes.push(u8::from(value));
}

fn inventory_push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn inventory_push_i32_option(bytes: &mut Vec<u8>, value: Option<i32>) {
    match value {
        Some(value) => {
            bytes.push(1);
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        None => bytes.push(0),
    }
}

fn inventory_push_hash_option(bytes: &mut Vec<u8>, value: Option<[u8; 32]>) {
    match value {
        Some(value) => {
            bytes.push(1);
            bytes.extend_from_slice(&value);
        }
        None => bytes.push(0),
    }
}

fn inventory_push_str(bytes: &mut Vec<u8>, value: &str) {
    inventory_push_u64(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

fn foreign_service_inventory_hash() -> [u8; 32] {
    let mut bytes = Vec::new();
    inventory_push_u64(&mut bytes, crate::service_inventory::SERVICES.len() as u64);

    let personal = crate::personal_shell_service::lifecycle_snapshot();
    inventory_push_bool(&mut bytes, personal.running);
    inventory_push_str(&mut bytes, personal.last_reason);

    match crate::echo_service::loaded_snapshot() {
        Some(echo) => {
            bytes.push(1);
            inventory_push_str(&mut bytes, crate::echo_service::health_state(echo));
            inventory_push_bool(&mut bytes, echo.running);
            inventory_push_u64(&mut bytes, echo.generation);
            inventory_push_u64(&mut bytes, echo.run_count);
            inventory_push_str(&mut bytes, echo.last_action);
        }
        None => bytes.push(0),
    }

    match crate::granted_candidate_service::loaded_snapshot() {
        Some(granted) => {
            bytes.push(1);
            inventory_push_str(
                &mut bytes,
                crate::granted_candidate_service::health_state(granted),
            );
            inventory_push_str(&mut bytes, granted.trust_tier);
            inventory_push_bool(&mut bytes, granted.running);
            inventory_push_u64(&mut bytes, granted.generation);
            inventory_push_str(&mut bytes, granted.last_run_outcome);
            inventory_push_str(&mut bytes, granted.last_action);
            inventory_push_bool(
                &mut bytes,
                crate::granted_candidate_service::service_slot_activation_active(granted),
            );
        }
        None => bytes.push(0),
    }

    let autoload = crate::project_app_autoload::snapshot();
    inventory_push_bool(&mut bytes, autoload.installed);
    let workspace_visible =
        crate::workspace_candidate_service::visible_in_inventory() || autoload.installed;
    inventory_push_bool(&mut bytes, workspace_visible);
    if workspace_visible {
        let workspace = crate::workspace_candidate_service::snapshot();
        inventory_push_str(
            &mut bytes,
            crate::workspace_candidate_service::health_state(workspace),
        );
        inventory_push_str(&mut bytes, workspace.phase);
        inventory_push_str(&mut bytes, workspace.last_reason);
        inventory_push_u64(&mut bytes, workspace.generation);
        inventory_push_u64(&mut bytes, workspace.run_count);
        inventory_push_i32_option(&mut bytes, workspace.last_return_value);
        inventory_push_str(&mut bytes, workspace.last_run_outcome);
        inventory_push_hash_option(
            &mut bytes,
            workspace.binding.map(|binding| binding.candidate_sha256),
        );
        inventory_push_hash_option(
            &mut bytes,
            workspace.binding.map(|binding| binding.receipt_sha256),
        );
    }

    match crate::hello_service::loaded_snapshot() {
        Some(hello) => {
            bytes.push(1);
            inventory_push_bool(&mut bytes, hello.running);
            inventory_push_u64(&mut bytes, hello.generation);
            inventory_push_str(&mut bytes, hello.last_action);
            bytes.extend_from_slice(&crate::hello_service::descriptor_source_hash(
                hello.load_descriptor,
            ));
            inventory_push_hash_option(
                &mut bytes,
                hello
                    .load_descriptor
                    .source_envelope
                    .map(|envelope| envelope.envelope_hash),
            );
            bytes.extend_from_slice(&crate::hello_service::artifact_load_plan_preflight_hash(
                hello.load_descriptor,
            ));
            bytes.extend_from_slice(&crate::hello_service::service_slot_activation_hash(
                hello.load_descriptor,
            ));
        }
        None => bytes.push(0),
    }

    sha256_bytes(&bytes)
}

pub(crate) fn emit_capability_selftest() {
    let controller = crate::pci::find_mass_storage_controller();
    let disk_before = controller.map(wasi_build_job::persist_region_hashes);

    let zero_grant_fresh = run_zero_import_fixture();
    let host_call = run_zero_grant_host_call_fixture();
    let peer_before = foreign_service_inventory_hash();
    let cross_service = run_cross_service_fixture();
    let peer_after = foreign_service_inventory_hash();

    let disk = wasi_build_job::compare_persist_region_hashes(controller, disk_before);
    let persistent_effect_free = match &disk {
        wasi_build_job::StorageSelftestDiskEvidence::Absent => true,
        wasi_build_job::StorageSelftestDiskEvidence::Present {
            reclog_unchanged,
            artstor_unchanged,
        } => *reclog_unchanged && *artstor_unchanged,
    };
    let logged = host_call.refusal_recorded && cross_service.refusal_recorded;
    // env.counter_get is reachable only through a linked host function. This
    // fixture has zero linked imports and stops before instantiation, so the
    // current-boot host counter cannot be called or changed.
    let host_effect_free = host_call.execution_effect_free;
    let peer_effect_free = cross_service.execution_effect_free && peer_before == peer_after;
    let pass = zero_grant_fresh
        && host_call.refused_before_instantiation
        && cross_service.refused_before_instantiation
        && logged
        && host_effect_free
        && peer_effect_free
        && persistent_effect_free;

    let line = alloc::format!(
        "RAIOS_CAPABILITY selftest={} zero_grant={} host_call={} cross_service={} logged={} host_effect={} peer_effect={} persistent_effect={}",
        if pass { "pass" } else { "fail" },
        if zero_grant_fresh { "fresh" } else { "failed" },
        host_call.token(),
        cross_service.token(),
        u8::from(logged),
        if host_effect_free { 0 } else { 1 },
        if peer_effect_free { 0 } else { 1 },
        if persistent_effect_free { 0 } else { 1 },
    );
    serial::write_raw_line(&line);

    let disk_line = match disk {
        wasi_build_job::StorageSelftestDiskEvidence::Absent => {
            String::from("RAIOS_CAPABILITY disk=absent")
        }
        wasi_build_job::StorageSelftestDiskEvidence::Present {
            reclog_unchanged,
            artstor_unchanged,
        } => alloc::format!(
            "RAIOS_CAPABILITY disk={} reclog_unchanged={} artstor_unchanged={}",
            if reclog_unchanged && artstor_unchanged {
                "pass"
            } else {
                "fail"
            },
            u8::from(reclog_unchanged),
            u8::from(artstor_unchanged),
        ),
    };
    serial::write_raw_line(&disk_line);
}
