use super::grant_table::{DurableImportState, GrantTable, HostImportId};
use super::import_gate::ImportGate;
use super::invocation::wasm_execution_busy;
use super::*;
use raios_core::host_import_abi_v1::HOST_IMPORT_ERROR_CAPABILITY_DENIED;

const MAX_WASM_LOG_BYTES: usize = 256;
// Keep in lockstep with the Phase-B guest buffer size.
pub(super) const MAX_WASM_INPUT_BYTES: usize = 4096;
const MAX_WASM_OUTPUT_BYTES: usize = 4096;
pub(super) const BUFFER_SERVICE_MAX_MEMORY_BYTES: usize = 2 * 1024 * 1024;
pub(super) const WASM_MEMORY_PAGE_BYTES: usize = 64 * 1024;

pub(super) const ZERO_SHA256: [u8; 32] = [0; 32];

static CURRENT_BOOT_COUNTER: Mutex<u64> = Mutex::new(0);
static NEXT_INSTANCE_GENERATION: AtomicU64 = AtomicU64::new(1);

pub(crate) struct EchoRunEvidence {
    pub(crate) validation_ok: bool,
    pub(crate) instantiation_ok: bool,
    pub(crate) run_outcome: &'static str,
    pub(crate) return_value: Option<i32>,
    pub(crate) fuel_budget: u64,
    pub(crate) fuel_used: u64,
    pub(crate) log_line: Option<String>,
    pub(crate) import_grant_performed: bool,
    pub(crate) import_grant_status: &'static str,
    pub(crate) import_grant_reason: &'static str,
    pub(crate) authorized_import_count: u64,
    pub(crate) authorized_import_list_sha256: [u8; 32],
    pub(crate) captured_output_len: u64,
    pub(crate) captured_output_sha256: [u8; 32],
    pub(crate) raw_captured_output: Vec<u8>,
    pub(crate) linked_host_import_count: u64,
    pub(crate) module_imports_within_authorized_list: bool,
    pub(crate) missing_import_module: Option<String>,
    pub(crate) missing_import_name: Option<String>,
}

#[derive(Clone, Copy)]
pub(crate) struct WorkspaceImportInspection {
    pub(crate) validation_ok: bool,
    pub(crate) import_list_observed: bool,
    pub(crate) import_count: usize,
    pub(crate) reason: &'static str,
}

pub(super) struct EnvelopeState {
    pub(super) log_line: Option<String>,
    staged_input: Vec<u8>,
    captured_output: Vec<u8>,
    pub(super) limits: StoreLimits,
    instance_generation: u64,
    grants: GrantTable,
}

impl EnvelopeState {
    pub(super) fn instance_generation(&self) -> u64 {
        self.instance_generation
    }

    pub(super) fn grant_import(&mut self, surface: HostImportId) -> bool {
        self.grants.grant(self.instance_generation, surface)
    }

    pub(super) fn revoke_import(&mut self, surface: HostImportId) -> bool {
        self.grants.revoke(self.instance_generation, surface)
    }

    pub(super) fn import_is_live(&self, surface: HostImportId) -> bool {
        self.grants.is_live(self.instance_generation, surface)
    }
}

pub(super) struct AuthorizedWasmImports<'a> {
    imports: &'a [(&'a str, &'a str)],
    decision: WasmImportGrantDecision,
    import_list_sha256: [u8; 32],
}

struct ImportGrantEvidence {
    performed: bool,
    status: &'static str,
    reason: &'static str,
    authorized_import_count: u64,
    authorized_import_list_sha256: [u8; 32],
    linked_host_import_count: u64,
    module_imports_within_authorized_list: bool,
    missing_import_module: Option<String>,
    missing_import_name: Option<String>,
}

pub(crate) struct NegativeRun {
    pub(crate) validation_ok: bool,
    pub(crate) instantiation_ok: bool,
    pub(crate) link_error_kind: &'static str,
    pub(crate) missing_import_module: Option<&'static str>,
    pub(crate) missing_import_name: Option<&'static str>,
    pub(crate) boundary_held: bool,
}

pub(super) fn execute_echo_module(validation_ok: bool) -> EchoRunEvidence {
    execute_validated_module_bytes(
        ECHO_WASM_ARTIFACT_BYTES,
        "raios_service_main",
        ECHO_SERVICE_ID,
        true,
        ECHO_AUTHORIZED_IMPORTS,
        validation_ok,
        &[],
        ECHO_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn execute_module_bytes(
    bytes: &[u8],
    entrypoint: &str,
    service_id: &str,
    artifact_sha256_present: bool,
    requested_imports: &[(&str, &str)],
) -> EchoRunEvidence {
    execute_validated_module_bytes(
        bytes,
        entrypoint,
        service_id,
        artifact_sha256_present,
        requested_imports,
        validate_module_bytes(bytes),
        &[],
        ECHO_WASM_FUEL_BUDGET,
    )
}

pub(crate) fn inspect_workspace_imports(bytes: &[u8]) -> WorkspaceImportInspection {
    let engine = metered_engine();
    let module = match Module::new(&engine, bytes) {
        Ok(module) => module,
        Err(_) => {
            return WorkspaceImportInspection {
                validation_ok: false,
                import_list_observed: false,
                import_count: 0,
                reason: "workspace_module_invalid",
            }
        }
    };
    let import_count = module.imports().count();
    WorkspaceImportInspection {
        validation_ok: true,
        import_list_observed: true,
        import_count,
        reason: if import_count == 0 {
            "workspace_import_surface_observed_empty"
        } else {
            "workspace_import_surface_not_empty"
        },
    }
}

pub(crate) fn execute_workspace_no_import_candidate(bytes: &[u8]) -> EchoRunEvidence {
    if wasm_execution_busy() {
        return wasm_busy_run(WORKSPACE_SERVICE_ID, WORKSPACE_FUEL_BUDGET);
    }
    let inspection = inspect_workspace_imports(bytes);
    let grant = evaluate_observed_wasm_import_grant(
        &WasmImportGrantInput {
            service_id: Some(WORKSPACE_SERVICE_ID),
            artifact_sha256_present: true,
            requested_imports: &[],
            policy_allows_beyond_env: false,
        },
        inspection.import_list_observed,
    );
    let import_evidence = ImportGrantEvidence {
        performed: grant.performed,
        status: grant.status,
        reason: grant.reason,
        authorized_import_count: grant.authorized_import_count as u64,
        authorized_import_list_sha256: authorized_import_list_sha256(WORKSPACE_SERVICE_ID, &[]),
        linked_host_import_count: 0,
        module_imports_within_authorized_list: inspection.import_count == 0,
        missing_import_module: None,
        missing_import_name: None,
    };
    if !inspection.validation_ok || inspection.import_count != 0 || !grant.performed {
        return positive_run(
            WORKSPACE_SERVICE_ID,
            WORKSPACE_FUEL_BUDGET,
            inspection.validation_ok,
            false,
            inspection.reason,
            None,
            0,
            None,
            import_evidence,
        );
    }
    let engine = metered_engine();
    let module = match Module::new(&engine, bytes) {
        Ok(module) => module,
        Err(_) => {
            return positive_run(
                WORKSPACE_SERVICE_ID,
                WORKSPACE_FUEL_BUDGET,
                false,
                false,
                "workspace_module_invalid",
                None,
                0,
                None,
                import_evidence,
            )
        }
    };
    let mut store = Store::new(&engine, limited_state(WORKSPACE_MEMORY_LIMIT_BYTES));
    store.limiter(|state| &mut state.limits);
    if store.add_fuel(WORKSPACE_FUEL_BUDGET).is_err() {
        return positive_run(
            WORKSPACE_SERVICE_ID,
            WORKSPACE_FUEL_BUDGET,
            true,
            false,
            "fuel_metering_unavailable",
            None,
            0,
            None,
            import_evidence,
        );
    }
    let linker = Linker::<EnvelopeState>::new(&engine);
    let instance = match linker.instantiate(&mut store, &module) {
        Ok(pre) => match pre.start(&mut store) {
            Ok(instance) => instance,
            Err(error) => {
                return positive_run(
                    WORKSPACE_SERVICE_ID,
                    WORKSPACE_FUEL_BUDGET,
                    true,
                    false,
                    classify_workspace_error(error),
                    None,
                    store.fuel_consumed().unwrap_or(0),
                    None,
                    import_evidence,
                )
            }
        },
        Err(error) => {
            return positive_run(
                WORKSPACE_SERVICE_ID,
                WORKSPACE_FUEL_BUDGET,
                true,
                false,
                classify_workspace_error(error),
                None,
                store.fuel_consumed().unwrap_or(0),
                None,
                import_evidence,
            )
        }
    };
    let Some(function) = instance
        .get_export(&store, WORKSPACE_ENTRYPOINT)
        .and_then(Extern::into_func)
    else {
        return positive_run(
            WORKSPACE_SERVICE_ID,
            WORKSPACE_FUEL_BUDGET,
            true,
            true,
            "entrypoint_missing",
            None,
            store.fuel_consumed().unwrap_or(0),
            None,
            import_evidence,
        );
    };
    let mut output = [Value::I32(0)];
    match function.call(&mut store, &[], &mut output) {
        Ok(()) => positive_run(
            WORKSPACE_SERVICE_ID,
            WORKSPACE_FUEL_BUDGET,
            true,
            true,
            if output[0].i32().is_some() {
                "success"
            } else {
                "entrypoint_type_mismatch"
            },
            output[0].i32(),
            store.fuel_consumed().unwrap_or(0),
            None,
            import_evidence,
        ),
        Err(error) => positive_run(
            WORKSPACE_SERVICE_ID,
            WORKSPACE_FUEL_BUDGET,
            true,
            true,
            classify_workspace_error(error),
            None,
            store.fuel_consumed().unwrap_or(0),
            None,
            import_evidence,
        ),
    }
}

fn classify_workspace_error(error: wasmi::Error) -> &'static str {
    match error {
        wasmi::Error::Trap(trap) => match trap.trap_code() {
            Some(TrapCode::OutOfFuel) => "fuel_exhausted",
            Some(TrapCode::UnreachableCodeReached) => "guest_trap",
            Some(_) => "guest_trap",
            None => "guest_trap",
        },
        wasmi::Error::Memory(_) => "memory_limit_exceeded",
        wasmi::Error::Instantiation(_) => "instantiation_failed",
        _ => "workspace_run_failed",
    }
}

pub(super) fn execute_validated_module_bytes(
    bytes: &[u8],
    entrypoint: &str,
    service_id: &str,
    artifact_sha256_present: bool,
    requested_imports: &[(&str, &str)],
    validation_ok: bool,
    staged_input: &[u8],
    fuel_budget: u64,
) -> EchoRunEvidence {
    if wasm_execution_busy() {
        return wasm_busy_run(service_id, fuel_budget);
    }
    let authorized =
        match authorize_wasm_imports(service_id, artifact_sha256_present, requested_imports) {
            Ok(authorized) => authorized,
            Err(decision) => {
                return positive_run(
                    service_id,
                    fuel_budget,
                    false,
                    false,
                    "import_grant_denied",
                    None,
                    0,
                    None,
                    import_grant_denied_evidence(decision),
                )
            }
        };
    if !validation_ok {
        return positive_run(
            service_id,
            fuel_budget,
            false,
            false,
            "validation_failed",
            None,
            0,
            None,
            import_grant_evidence(&authorized, 0, false, None),
        );
    }

    let wasm = Vec::from(bytes).into_boxed_slice();
    let engine = metered_engine();
    let module = match Module::new(&engine, &*wasm) {
        Ok(module) => Box::new(module),
        Err(_) => {
            return positive_run(
                service_id,
                fuel_budget,
                true,
                false,
                "module_compile_failed",
                None,
                0,
                None,
                import_grant_evidence(&authorized, 0, false, None),
            )
        }
    };
    let mut store = Box::new(Store::new(&engine, buffer_state(staged_input)));
    store.limiter(|state| &mut state.limits);
    if store.add_fuel(fuel_budget).is_err() {
        return positive_run(
            service_id,
            fuel_budget,
            true,
            false,
            "fuel_metering_unavailable",
            None,
            0,
            None,
            import_grant_evidence(&authorized, 0, false, None),
        );
    }
    let mut linker = Box::new(Linker::<EnvelopeState>::new(&engine));
    let linked_host_import_count = match define_granted_imports(&mut linker, &authorized) {
        Ok(count) => count,
        Err(reason) => {
            return positive_run(
                service_id,
                fuel_budget,
                true,
                false,
                reason,
                None,
                0,
                None,
                import_grant_evidence(&authorized, 0, false, None),
            )
        }
    };
    if let Some(missing) = first_unauthorized_module_import(&module, &authorized) {
        return positive_run(
            service_id,
            fuel_budget,
            true,
            false,
            "module_import_not_authorized",
            None,
            0,
            None,
            import_grant_evidence(&authorized, linked_host_import_count, false, Some(missing)),
        );
    }

    let instance = match linker.instantiate(&mut *store, &module) {
        Ok(instance) => match instance.start(&mut *store) {
            Ok(instance) => instance,
            Err(_) => {
                return positive_run(
                    service_id,
                    fuel_budget,
                    true,
                    false,
                    "instantiation_start_trap",
                    None,
                    store.fuel_consumed().unwrap_or(0),
                    store.data().log_line.clone(),
                    import_grant_evidence(&authorized, linked_host_import_count, true, None),
                )
            }
        },
        Err(_) => {
            return positive_run(
                service_id,
                fuel_budget,
                true,
                false,
                "instantiation_failed",
                None,
                store.fuel_consumed().unwrap_or(0),
                store.data().log_line.clone(),
                import_grant_evidence(&authorized, linked_host_import_count, true, None),
            )
        }
    };

    let Some(func) = instance
        .get_export(&*store, entrypoint)
        .and_then(Extern::into_func)
    else {
        return positive_run(
            service_id,
            fuel_budget,
            true,
            true,
            "entrypoint_missing",
            None,
            store.fuel_consumed().unwrap_or(0),
            store.data().log_line.clone(),
            import_grant_evidence(&authorized, linked_host_import_count, true, None),
        );
    };

    let mut outputs = Vec::from([Value::I32(0)]).into_boxed_slice();
    match func.call(&mut *store, &[], &mut outputs) {
        Ok(()) => {
            let return_value = outputs[0].i32();
            let outcome = if return_value.is_some() {
                "success"
            } else {
                "bad_return_type"
            };
            let mut ev = positive_run(
                service_id,
                fuel_budget,
                true,
                true,
                outcome,
                return_value,
                store.fuel_consumed().unwrap_or(0),
                store.data().log_line.clone(),
                import_grant_evidence(&authorized, linked_host_import_count, true, None),
            );
            let out = &store.data().captured_output;
            if out.is_empty() {
                ev.captured_output_len = 0;
                ev.captured_output_sha256 = ZERO_SHA256;
            } else {
                ev.captured_output_len = out.len() as u64;
                ev.captured_output_sha256 = sha256_bytes(out);
            }
            ev.raw_captured_output = out.clone();
            ev
        }
        Err(_) => {
            let mut ev = positive_run(
                service_id,
                fuel_budget,
                true,
                true,
                "trap",
                None,
                store.fuel_consumed().unwrap_or(0),
                store.data().log_line.clone(),
                import_grant_evidence(&authorized, linked_host_import_count, true, None),
            );
            let out = &store.data().captured_output;
            if out.is_empty() {
                ev.captured_output_len = 0;
                ev.captured_output_sha256 = ZERO_SHA256;
            } else {
                ev.captured_output_len = out.len() as u64;
                ev.captured_output_sha256 = sha256_bytes(out);
            }
            ev.raw_captured_output = out.clone();
            ev
        }
    }
}

fn positive_run(
    service_id: &str,
    fuel_budget: u64,
    validation_ok: bool,
    instantiation_ok: bool,
    run_outcome: &'static str,
    return_value: Option<i32>,
    fuel_used: u64,
    log_line: Option<String>,
    import_grant: ImportGrantEvidence,
) -> EchoRunEvidence {
    let evidence = EchoRunEvidence {
        validation_ok,
        instantiation_ok,
        run_outcome,
        return_value,
        fuel_budget,
        fuel_used,
        log_line,
        import_grant_performed: import_grant.performed,
        import_grant_status: import_grant.status,
        import_grant_reason: import_grant.reason,
        authorized_import_count: import_grant.authorized_import_count,
        authorized_import_list_sha256: import_grant.authorized_import_list_sha256,
        captured_output_len: 0,
        captured_output_sha256: ZERO_SHA256,
        raw_captured_output: Vec::new(),
        linked_host_import_count: import_grant.linked_host_import_count,
        module_imports_within_authorized_list: import_grant.module_imports_within_authorized_list,
        missing_import_module: import_grant.missing_import_module,
        missing_import_name: import_grant.missing_import_name,
    };
    let _ = (
        evidence.captured_output_len,
        evidence.captured_output_sha256,
    );
    emit_import_grant_marker(service_id, &evidence);
    evidence
}

pub(super) fn wasm_busy_run(service_id: &str, fuel_budget: u64) -> EchoRunEvidence {
    positive_run(
        service_id,
        fuel_budget,
        false,
        false,
        "wasm_execution_busy",
        None,
        0,
        None,
        ImportGrantEvidence {
            performed: false,
            status: "import_grant_not_evaluated",
            reason: "wasm_execution_busy",
            authorized_import_count: 0,
            authorized_import_list_sha256: ZERO_SHA256,
            linked_host_import_count: 0,
            module_imports_within_authorized_list: false,
            missing_import_module: None,
            missing_import_name: None,
        },
    )
}

pub(super) fn metered_engine() -> Box<Engine> {
    let mut config = Config::default();
    config.consume_fuel(true);
    Box::new(Engine::new(&config))
}

pub(super) fn default_state() -> EnvelopeState {
    new_state(Vec::new(), StoreLimitsBuilder::new().build(), None)
}

fn next_instance_generation() -> u64 {
    NEXT_INSTANCE_GENERATION
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |generation| {
            generation.checked_add(1)
        })
        .expect("Wasm instance generation space exhausted")
}

fn new_state(
    staged_input: Vec<u8>,
    limits: StoreLimits,
    durable_domain: Option<(&str, u64, [u8; 32])>,
) -> EnvelopeState {
    let instance_generation = next_instance_generation();
    let mut grants = GrantTable::new();
    // Slice 2 preserves existing env.counter_get behavior for the envelope
    // constructors. The linker remains the grant-list boundary; this live
    // slot adds revocable indirection only after the import was linked.
    let _ = grants.grant(instance_generation, HostImportId::EnvCounterGet);
    if let Some((service_id, domain_instance, binding_sha256)) = durable_domain {
        match super::grant_table::durable_import_state(
            service_id,
            domain_instance,
            binding_sha256,
            HostImportId::EnvCounterGet,
        ) {
            DurableImportState::Revoked | DurableImportState::DeniedInvalidProjection => {
                // Materialize the durable boot fold into the exact table that
                // host_counter_get consults immediately before its effect.
                let _ = grants.revoke(instance_generation, HostImportId::EnvCounterGet);
            }
            DurableImportState::LegacyDefault | DurableImportState::Live => {}
        }
    }
    EnvelopeState {
        log_line: None,
        staged_input,
        captured_output: Vec::new(),
        limits,
        instance_generation,
        grants,
    }
}

pub(super) fn limited_state(memory_size: usize) -> EnvelopeState {
    new_state(
        Vec::new(),
        StoreLimitsBuilder::new()
            .memory_size(memory_size)
            .instances(1)
            .memories(1)
            .tables(0)
            .build(),
        None,
    )
}

pub(super) fn limited_state_for_durable_domain(
    memory_size: usize,
    service_id: &str,
    domain_instance: u64,
    binding_sha256: [u8; 32],
) -> EnvelopeState {
    new_state(
        Vec::new(),
        StoreLimitsBuilder::new()
            .memory_size(memory_size)
            .instances(1)
            .memories(1)
            .tables(0)
            .build(),
        Some((service_id, domain_instance, binding_sha256)),
    )
}

fn buffer_state(staged_input: &[u8]) -> EnvelopeState {
    new_state(
        staged_input.to_vec(),
        StoreLimitsBuilder::new()
            .memory_size(BUFFER_SERVICE_MAX_MEMORY_BYTES)
            .instances(1)
            .memories(1)
            // Measured 2026-07-14: httphead/certspki instantiate one funcref
            // table (3 and 2 elements), bufecho/certwindow none — `.tables(0)`
            // would deny those signed guests before instantiation. Bound is the
            // measured shape plus an element cap against table.grow exhaustion.
            .tables(1)
            .table_elements(64)
            .build(),
        None,
    )
}

pub(super) fn authorize_wasm_imports<'a>(
    service_id: &'a str,
    artifact_sha256_present: bool,
    requested_imports: &'a [(&'a str, &'a str)],
) -> Result<AuthorizedWasmImports<'a>, WasmImportGrantDecision> {
    let input = WasmImportGrantInput {
        service_id: Some(service_id),
        artifact_sha256_present,
        requested_imports,
        policy_allows_beyond_env: false,
    };
    let decision = evaluate_wasm_import_grant(&input);
    if !decision.performed {
        return Err(decision);
    }
    Ok(AuthorizedWasmImports {
        imports: requested_imports,
        decision,
        import_list_sha256: authorized_import_list_sha256(service_id, requested_imports),
    })
}

pub(super) fn define_granted_imports(
    linker: &mut Linker<EnvelopeState>,
    authorized: &AuthorizedWasmImports<'_>,
) -> Result<u64, &'static str> {
    let mut gate = ImportGate::new(linker);
    let mut idx = 0usize;
    while idx < authorized.imports.len() {
        match authorized.imports[idx] {
            ("env", "log") => {
                gate.link("env", "log", host_log)?;
            }
            ("env", "counter_get") => {
                gate.link("env", "counter_get", host_counter_get)?;
            }
            ("env", "input_len") => {
                gate.link("env", "input_len", host_input_len)?;
            }
            ("env", "input_read") => {
                gate.link("env", "input_read", host_input_read)?;
            }
            ("env", "output_write") => {
                gate.link("env", "output_write", host_output_write)?;
            }
            _ => return Err("missing_host_import_implementation"),
        }
        idx += 1;
    }
    Ok(gate.linked_count())
}

pub(super) fn first_unauthorized_module_import(
    module: &Module,
    authorized: &AuthorizedWasmImports<'_>,
) -> Option<(String, String)> {
    for import in module.imports() {
        if !authorized_contains(authorized, import.module(), import.name()) {
            return Some((String::from(import.module()), String::from(import.name())));
        }
    }
    None
}

fn authorized_contains(authorized: &AuthorizedWasmImports<'_>, module: &str, name: &str) -> bool {
    authorized
        .imports
        .iter()
        .any(|(authorized_module, authorized_name)| {
            *authorized_module == module && *authorized_name == name
        })
}

fn import_grant_denied_evidence(decision: WasmImportGrantDecision) -> ImportGrantEvidence {
    ImportGrantEvidence {
        performed: decision.performed,
        status: decision.status,
        reason: decision.reason,
        authorized_import_count: decision.authorized_import_count as u64,
        authorized_import_list_sha256: ZERO_SHA256,
        linked_host_import_count: 0,
        module_imports_within_authorized_list: false,
        missing_import_module: None,
        missing_import_name: None,
    }
}

fn import_grant_evidence(
    authorized: &AuthorizedWasmImports<'_>,
    linked_host_import_count: u64,
    module_imports_within_authorized_list: bool,
    missing_import: Option<(String, String)>,
) -> ImportGrantEvidence {
    let (missing_import_module, missing_import_name) = match missing_import {
        Some((module, name)) => (Some(module), Some(name)),
        None => (None, None),
    };
    ImportGrantEvidence {
        performed: authorized.decision.performed,
        status: authorized.decision.status,
        reason: authorized.decision.reason,
        authorized_import_count: authorized.decision.authorized_import_count as u64,
        authorized_import_list_sha256: authorized.import_list_sha256,
        linked_host_import_count,
        module_imports_within_authorized_list,
        missing_import_module,
        missing_import_name,
    }
}

fn emit_import_grant_marker(service_id: &str, evidence: &EchoRunEvidence) {
    let mut line = String::new();
    let _ = write!(
        &mut line,
        "WASM_IMPORT_GRANT {{\"service_id\":\"{}\",\"import_grant_performed\":{},\"import_grant_status\":\"{}\",\"import_grant_reason\":\"{}\",\"authorized_import_count\":{},\"authorized_import_list_sha256\":\"sha256:",
        service_id,
        if evidence.import_grant_performed { "true" } else { "false" },
        evidence.import_grant_status,
        evidence.import_grant_reason,
        evidence.authorized_import_count
    );
    push_sha256_hex(&mut line, evidence.authorized_import_list_sha256);
    let _ = write!(
        &mut line,
        "\",\"linked_host_import_count\":{},\"module_imports_within_authorized_list\":{},\"run_outcome\":\"{}\",\"missing_import_module\":",
        evidence.linked_host_import_count,
        if evidence.module_imports_within_authorized_list {
            "true"
        } else {
            "false"
        },
        evidence.run_outcome
    );
    push_json_string_or_null(&mut line, evidence.missing_import_module.as_deref());
    line.push_str(",\"missing_import_name\":");
    push_json_string_or_null(&mut line, evidence.missing_import_name.as_deref());
    line.push('}');
    serial::write_raw_line(&line);
}

fn push_json_string_or_null(out: &mut String, value: Option<&str>) {
    let Some(value) = value else {
        out.push_str("null");
        return;
    };
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}

fn push_sha256_hex(out: &mut String, value: [u8; 32]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut idx = 0usize;
    while idx < value.len() {
        let byte = value[idx];
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
        idx += 1;
    }
}

fn host_log(mut caller: Caller<'_, EnvelopeState>, ptr: i32, len: i32) -> Result<(), Trap> {
    caller
        .consume_fuel(25)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    if ptr < 0 || len < 0 {
        return Err(Trap::new("env.log negative pointer or length"));
    }
    let ptr = ptr as usize;
    let len = len as usize;
    if len > MAX_WASM_LOG_BYTES {
        return Err(Trap::new("env.log length exceeds 256 bytes"));
    }
    ptr.checked_add(len)
        .ok_or_else(|| Trap::new("env.log pointer overflow"))?;

    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| Trap::new("env.log memory export missing"))?;
    let mut bytes = Vec::new();
    bytes.resize(len, 0);
    memory
        .read(&caller, ptr, &mut bytes)
        .map_err(|_| Trap::from(TrapCode::MemoryOutOfBounds))?;

    let mut line = String::new();
    let mut idx = 0usize;
    while idx < bytes.len() {
        let byte = bytes[idx];
        line.push(if (0x20..=0x7e).contains(&byte) {
            byte as char
        } else {
            ' '
        });
        idx += 1;
    }

    serial::write_raw_str("WASM_GUEST_LOG ");
    serial::write_raw_line(&line);
    caller.data_mut().log_line = Some(line);
    Ok(())
}

fn host_input_len(mut caller: Caller<'_, EnvelopeState>) -> Result<i32, Trap> {
    caller
        .consume_fuel(5)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    let len = caller.data().staged_input.len();
    if len > MAX_WASM_INPUT_BYTES {
        return Err(Trap::new("env.input_len staged input exceeds max"));
    }
    Ok(len as i32)
}

fn host_input_read(mut caller: Caller<'_, EnvelopeState>, ptr: i32, len: i32) -> Result<i32, Trap> {
    caller
        .consume_fuel(25)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    if ptr < 0 || len < 0 {
        return Err(Trap::new("env.input_read negative pointer or length"));
    }
    let ptr = ptr as usize;
    let len = len as usize;
    if len > MAX_WASM_INPUT_BYTES {
        return Err(Trap::new("env.input_read length exceeds 4096 bytes"));
    }
    let count = len.min(caller.data().staged_input.len());
    ptr.checked_add(count)
        .ok_or_else(|| Trap::new("env.input_read pointer overflow"))?;

    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| Trap::new("env.input_read memory export missing"))?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&caller.data().staged_input[..count]);
    memory
        .write(&mut caller, ptr, &bytes)
        .map_err(|_| Trap::from(TrapCode::MemoryOutOfBounds))?;
    Ok(count as i32)
}

// Raw guest bytes: surfaced only as len+sha256 later. This env-module output
// channel is bounded + per-service-declared + subset-checked, and by design does
// NOT arm the policy_allows_beyond_env owner gate; move output_write to its own
// module if the owner later wants policy-gated output.
fn host_output_write(
    mut caller: Caller<'_, EnvelopeState>,
    ptr: i32,
    len: i32,
) -> Result<i32, Trap> {
    caller
        .consume_fuel(25)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    if ptr < 0 || len < 0 {
        return Err(Trap::new("env.output_write negative pointer or length"));
    }
    let ptr = ptr as usize;
    let len = len as usize;
    if len > MAX_WASM_OUTPUT_BYTES {
        return Err(Trap::new("env.output_write length exceeds 4096 bytes"));
    }
    ptr.checked_add(len)
        .ok_or_else(|| Trap::new("env.output_write pointer overflow"))?;

    let memory = caller
        .get_export("memory")
        .and_then(Extern::into_memory)
        .ok_or_else(|| Trap::new("env.output_write memory export missing"))?;
    let mut bytes = Vec::new();
    bytes.resize(len, 0);
    memory
        .read(&caller, ptr, &mut bytes)
        .map_err(|_| Trap::from(TrapCode::MemoryOutOfBounds))?;
    caller
        .data()
        .captured_output
        .len()
        .checked_add(len)
        .filter(|total| *total <= MAX_WASM_OUTPUT_BYTES)
        .ok_or_else(|| Trap::new("env.output_write captured output exceeds 4096 bytes"))?;
    caller.data_mut().captured_output.extend_from_slice(&bytes);
    Ok(len as i32)
}

fn host_counter_get(mut caller: Caller<'_, EnvelopeState>) -> Result<i64, Trap> {
    caller
        .consume_fuel(5)
        .map_err(|_| Trap::from(TrapCode::OutOfFuel))?;
    // env.counter_get is synchronous: this single bounded lookup and the
    // counter effect below have no suspension point between them.
    if !caller.data().import_is_live(HostImportId::EnvCounterGet) {
        return Ok(HOST_IMPORT_ERROR_CAPABILITY_DENIED as i64);
    }
    let mut counter = CURRENT_BOOT_COUNTER.lock();
    *counter = counter.saturating_add(1);
    Ok((*counter).min(i64::MAX as u64) as i64)
}

pub(super) fn current_boot_counter() -> u64 {
    *CURRENT_BOOT_COUNTER.lock()
}
